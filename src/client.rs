use std::{
    pin::Pin,
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};

use bytes::Bytes;
use ethers_core::types::{BlockId, BlockNumber, TransactionReceipt};
use ethers_providers::Middleware;
use eyre::Result;
use tokio::task::JoinHandle;

use crate::{
    channels::ChannelManager, config::Config, driver::Driver, rollup::RollupNode,
    transactions::TransactionManager,
};

/// Archon
///
/// This is the primary Archon client, responsible for orchestrating the batch submission pipeline.
///
/// Archon batching stages are broken up into actors, spawned in separate [std::thread::Thread]s.
///
/// The first actor is the [Driver]. The [Driver] polls an L1 [ethers_providers::Provider] for the
/// latest block on a given interval. It takes the [ethers_core::types::Block] and constructs an
/// [ethers_core::types::BlockId] which it then sends back to [Archon].
///
/// When [Archon] receives a [ethers_core::types::BlockId] from the [Driver], it passes it along to
/// the [ChannelManager]
#[derive(Debug, Default)]
pub struct Archon {
    // TODO: only store config params needed. Should build an archon instance from the Config object
    // TODO: eg: Archon::from(config)
    /// The inner [Config], used to configure [Archon]'s parameters
    config: Config,
    /// The inner [Driver]
    driver: Option<Driver>,
    /// A join handle on the driver
    driver_handle: Option<JoinHandle<Result<()>>>,
    /// The internal [Driver] receiver
    driver_receiver: Option<Receiver<Pin<Box<BlockId>>>>,
    /// The inner [ChannelManager]
    channel_manager: Option<ChannelManager>,
    /// A join handle on the [ChannelManager]
    channel_manager_handle: Option<JoinHandle<Result<()>>>,
    /// The internal [ChannelManager] receiver
    channel_manager_receiver: Option<Receiver<Pin<Box<Bytes>>>>,
    /// The internal [ChannelManager] sender
    channel_manager_sender: Option<Sender<Pin<Box<BlockId>>>>,
    /// The inner [TransactionManager]
    tx_manager: Option<TransactionManager>,
    /// A join handle on the [TransactionManager]
    tx_manager_handle: Option<JoinHandle<Result<()>>>,
    /// The internal [TransactionManager] sender
    tx_manager_sender: Option<Sender<Pin<Box<Bytes>>>>,
    /// The internal [TransactionManager] receiver
    tx_manager_receiver: Option<Receiver<Pin<Box<TransactionReceipt>>>>,
    /// The last stored [BlockId]
    last_stored_block: Option<BlockId>,
}

impl Archon {
    /// Constructs a new Archon instance from an optional [Config]
    pub fn new(config: Option<Config>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            ..Self::default()
        }
    }

    /// Sets the [Driver] instance on the [Archon] client
    pub fn with_driver(&mut self, driver: Driver) -> &mut Self {
        self.driver = Some(driver);
        self
    }

    /// Sets the [ChannelManager] instance on the [Archon] client
    pub fn with_channel_manager(&mut self, manager: ChannelManager) -> &mut Self {
        self.channel_manager = Some(manager);
        self
    }

    /// Sets the [TransactionManager] instance on the [Archon] client
    pub fn with_transaction_manager(&mut self, manager: TransactionManager) -> &mut Self {
        self.tx_manager = Some(manager);
        self
    }

    /// Instantiates a [Driver] if needed.
    /// Opens up a [std::sync::mpsc::channel] with the created [Driver].
    /// Spawns the [Driver] in a new [std::thread::Thread].
    ///
    /// Returns a [JoinHandle] to the spawned [Driver] if successfully spawed.
    pub fn spawn_driver(&mut self) -> Result<JoinHandle<Result<()>>> {
        let (sender, receiver) = mpsc::channel::<Pin<Box<BlockId>>>();
        self.driver_receiver = Some(receiver);
        let driver = self.driver.take();
        let driver = if let Some(mut d) = driver {
            d.with_channel(sender);
            d
        } else {
            // Construct an L1 client
            let l1_client = self.config.get_l1_client()?;
            let poll_interval = self.config.polling_interval;
            Driver::new(l1_client, poll_interval, Some(sender))
        };
        driver.spawn()
    }

    /// Instantiates a [ChannelManager] if needed.
    /// Opens up two [std::sync::mpsc::channel]s with the created [ChannelManager].
    /// One to send [BlockId]s to the [ChannelManager], and one to receive [Bytes].
    /// Spawns the [ChannelManager] in a new [std::thread::Thread].
    ///
    /// Returns a [JoinHandle] to the spawned [ChannelManager] if successfully spawed.
    pub fn spawn_channel_manager(&mut self) -> Result<JoinHandle<Result<()>>> {
        let (cm_sender, archon_receiver) = mpsc::channel::<Pin<Box<Bytes>>>();
        let (archon_sender, cm_receiver) = mpsc::channel::<Pin<Box<BlockId>>>();
        self.channel_manager_sender = Some(archon_sender);
        self.channel_manager_receiver = Some(archon_receiver);
        let channel_manager = self.channel_manager.take();
        let mut channel_manager = channel_manager.unwrap_or_default();
        channel_manager.with_sender(cm_sender);
        channel_manager.with_receiver(cm_receiver);
        channel_manager.spawn()
    }

    /// Instantiates a [TransactionManager] if needed.
    /// Opens up two [std::sync::mpsc::channel]s with the created [TransactionManager].
    /// One to send [Bytes]s to the [TransactionManager], and one to receive [TransactionReceipt]s.
    /// Spawns the [TransactionManager] in a new [std::thread::Thread].
    ///
    /// Returns a [JoinHandle] to the spawned [TransactionManager] if successfully spawed.
    pub fn spawn_transaction_manager(&mut self) -> Result<JoinHandle<Result<()>>> {
        let (tx_mgr_sender, archon_receiver) = mpsc::channel::<Pin<Box<TransactionReceipt>>>();
        let (archon_sender, tx_mgr_receiver) = mpsc::channel::<Pin<Box<Bytes>>>();
        self.tx_manager_sender = Some(archon_sender);
        self.tx_manager_receiver = Some(archon_receiver);
        let transaction_manager = self.tx_manager.take();
        let mut transaction_manager = transaction_manager.unwrap_or(TransactionManager::new(
            Some(self.config.network.into()),
            Some(self.config.batcher_inbox),
            Some(self.config.proposer_address),
            Some(self.config.batcher_private_key.clone()),
            self.config.get_l1_client()?,
        ));
        transaction_manager.with_sender(tx_mgr_sender);
        transaction_manager.with_receiver(tx_mgr_receiver);
        transaction_manager.spawn()
    }

    /// Runs [Archon]'s batch submission pipeline.
    pub async fn start(&mut self) -> Result<()> {
        tracing::info!(target: "archon", "Starting batch submission pipeline...");

        // Build and spawn a driver
        let driver_handle = self.spawn_driver()?;
        self.driver_handle = Some(driver_handle);

        // Build and spawn a channel manager
        let channel_manager_handle = self.spawn_channel_manager()?;
        self.channel_manager_handle = Some(channel_manager_handle);

        // Build and spawn a transaction manager
        let tx_manager_handle = self.spawn_transaction_manager()?;
        self.tx_manager_handle = Some(tx_manager_handle);

        // Loads all blocks since the previous stored block
        // 1. Fetch the sync status of the sequencer
        // 2. Check if the sync status is valid or if we are all the way up to date
        // 3. Check if it needs to initialize state OR it is lagging (todo: lagging just means race condition?)
        // 4. Load all new blocks into the local state.
        // TODO: refactor this in an l2 driver
        // TODO: should mirror: https://github.com/ethereum-optimism/optimism/blob/develop/op-batcher/batcher/driver.go#L272
        tracing::info!(target: "archon", "Listening to L2 Blocks...");
        let rollup = RollupNode::new(&self.config.l2_client_rpc_url)?;
        let interval = self
            .config
            .polling_interval
            .unwrap_or(Duration::from_secs(4));
        loop {
            // Await the poll interval at the loop start so we can ergonomically continue below.
            std::thread::sleep(interval);

            // Fetch the [SyncStatus] of the rollup node
            let sync_status = if let Ok(s) = rollup.sync_status().await {
                s
            } else {
                continue;
            };

            // If the l1 head is empty, the sync status is invalid.
            if sync_status.head_l1 == 0 {
                tracing::warn!(target: "archon", "Invalid sync status: {:?}", sync_status);
                continue;
            }

            // Check last stored to see if it needs to be set on startup OR set if is lagged behind.
            // It lagging implies that the op-node processed some batches that were submitted prior
            // to the current instance of the batcher being alive.
            if self.last_stored_block.is_none() {
                tracing::debug!(target: "archon", "Starting batch-submitter work at safe-head {}", sync_status.safe_l2);
                self.last_stored_block =
                    Some(BlockId::Number(BlockNumber::from(sync_status.safe_l2)));
            } else if let Some(BlockId::Number(BlockNumber::Number(lsb))) = self.last_stored_block {
                if lsb < sync_status.safe_l2.into() {
                    tracing::debug!(target: "archon", "Last stored block lagged behind L2 safe head: batch submission will continue from the safe head now");
                    self.last_stored_block =
                        Some(BlockId::Number(BlockNumber::from(sync_status.safe_l2)));
                }
            }

            // Check if we should even attempt to load any blocks.
            if sync_status.safe_l2 >= sync_status.unsafe_l2 {
                tracing::warn!(target: "archon", "L2 safe head ahead of L2 unsafe head: {:?}", sync_status);
                continue;
            }

            // Use the potentially updated last stored block as the start
            let start_block = if let Some(BlockId::Number(BlockNumber::Number(lsb))) =
                self.last_stored_block
            {
                lsb
            } else {
                tracing::warn!(target: "archon", "Last stored block is None: this should never happen");
                continue;
            };

            // Use the [SyncStatus] unsafe L2 head as the end
            let end_block = sync_status.unsafe_l2;

            // Load all blocks
            for i in start_block.as_u64()..end_block {
                match self.load_block_into_state(i).await {
                    Ok(block_id) => {
                        self.last_stored_block = Some(block_id);
                    }
                    Err(e) => {
                        tracing::warn!(target: "archon", "Failed to load block into state: {:?}", e);
                        break;
                    }
                }
            }
            tracing::debug!(target: "archon", "Loaded blocks into state: {}..{}", start_block, end_block);
        }

        // TODO: Construct a new sequencer "service" to feed into the batch driver
        // TODO: Construct a new proposer "service" to feed into the batch driver
    }

    /// Fetches & stores a single [Block] into `state`.
    /// Returns the [BlockId] it loaded.
    pub async fn load_block_into_state(&self, block_number: u64) -> Result<BlockId> {
        let l2_client = self.config.get_l2_client()?;
        let _block = l2_client.get_block(block_number).await?;
        // TODO: push this block to the channel manager
        tracing::info!(target: "archon", "Forwarded L2 block to the channel manager: {:?}", block_number);
        Ok(BlockId::Number(BlockNumber::Number(block_number.into())))
    }
}
