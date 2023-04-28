use std::{
    pin::Pin,
    sync::mpsc::{
        self,
        Receiver,
        Sender,
    },
    time::Duration,
};

use bytes::Bytes;
use ethers_core::types::{
    BlockId,
    TransactionReceipt,
};
use eyre::Result;
use tokio::task::JoinHandle;

use crate::{
    channels::ChannelManager,
    config::Config,
    driver::Driver,
    metrics::Metrics,
    pipeline_builder::PipelineBuilder,
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
/// the [ChannelManager].
#[derive(Debug, Default)]
pub struct Archon {
    /// The inner [Config], used to configure [Archon]'s parameters
    config: Config,
    /// The inner [Driver]
    driver: Option<Driver>,
    /// A join handle on the driver
    driver_handle: Option<JoinHandle<Result<()>>>,
    /// Driver receiver
    driver_receiver: Option<Receiver<Pin<Box<BlockId>>>>,
    /// The inner [ChannelManager]
    channel_manager: Option<ChannelManager>,
    /// A join handle on the [ChannelManager]
    channel_manager_handle: Option<JoinHandle<Result<()>>>,
    /// A join handle on the [ChannelManager] block processor
    channel_manager_block_handle: Option<JoinHandle<Result<()>>>,
    /// The internal [ChannelManager] sender
    channel_manager_sender: Option<Sender<Pin<Box<BlockId>>>>,
    /// The inner [TransactionManager]
    tx_manager: Option<TransactionManager>,
    /// A join handle on the [TransactionManager]
    tx_manager_handle: Option<JoinHandle<Result<()>>>,
    /// The internal [TransactionManager] sender
    tx_manager_sender: Option<Sender<Pin<Box<Bytes>>>>,
    /// Transaction manager receiver
    tx_manager_receiver: Option<Receiver<Pin<Box<TransactionReceipt>>>>,
    /// A metrics server for the [Archon] client
    metrics: Option<Metrics>,
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

    /// Sets a [Metrics] server on the [Archon] client
    pub fn with_metrics(&mut self, metrics: Metrics) -> &mut Self {
        self.metrics = Some(metrics);
        self
    }

    /// Returns a reference to [Config]
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Sets the internal [TransactionManager] sender
    pub fn with_tx_manager_sender(
        &mut self,
        sender: Sender<Pin<Box<Bytes>>>,
    ) -> &mut Self {
        self.tx_manager_sender = Some(sender);
        self
    }

    /// Instantiates a [Driver] if needed.
    /// Opens up a [std::sync::mpsc::channel] with the created [Driver].
    /// Spawns the [Driver] in a new [std::thread::Thread].
    ///
    /// Returns a [JoinHandle] to the spawned [Driver] if successfully spawed.
    pub fn spawn_driver(&mut self) -> Result<()> {
        let (sender, receiver) = mpsc::channel::<Pin<Box<BlockId>>>();
        self.driver_receiver = Some(receiver);
        let driver = self.driver.take();
        let mut driver = if let Some(d) = driver {
            d
        } else {
            // Construct an L1 client
            let l1_client = self.config.get_l1_client()?;
            let poll_interval = self.config.polling_interval;
            Driver::new(l1_client, poll_interval, None)
        };
        driver.with_channel(sender);
        self.driver_handle = Some(
            driver
                .spawn()
                .map_err(|_| eyre::eyre!("Failed to spawn driver"))?,
        );
        Ok(())
    }

    /// Instantiates a [ChannelManager] if needed.
    /// Opens up two [std::sync::mpsc::channel]s with the created [ChannelManager].
    /// One to send [BlockId]s to the [ChannelManager], and one to receive [Bytes].
    /// Spawns the [ChannelManager] in a new [std::thread::Thread].
    ///
    /// Returns a [JoinHandle] to the spawned [ChannelManager] if successfully spawed.
    pub fn spawn_channel_manager(&mut self) -> Result<()> {
        let (cm_sender, _) = mpsc::channel::<Pin<Box<Bytes>>>();
        let (archon_sender, cm_receiver) = mpsc::channel::<Pin<Box<BlockId>>>();
        self.channel_manager_sender = Some(archon_sender);
        // self.channel_manager_receiver = Some(archon_receiver);
        let channel_manager = self.channel_manager.take();
        let mut channel_manager = channel_manager.unwrap_or_default();
        channel_manager.with_sender(cm_sender);
        channel_manager.with_receiver(cm_receiver);
        let poll_interval = self
            .config
            .polling_interval
            .unwrap_or(Duration::from_secs(5));
        self.channel_manager_block_handle = Some(
            channel_manager
                .spawn_block_processor(
                    &self.config.rollup_node_rpc_url,
                    &self.config.l2_client_rpc_url,
                    poll_interval,
                )
                .map_err(|_| {
                    eyre::eyre!("Failed to spawn channel manager block handler")
                })?,
        );
        self.channel_manager_handle = Some(
            channel_manager
                .spawn()
                .map_err(|_| eyre::eyre!("Failed to spawn channel manager"))?,
        );
        Ok(())
    }

    /// Instantiates a [TransactionManager] if needed.
    /// Opens up two [std::sync::mpsc::channel]s with the created [TransactionManager].
    /// One to send [Bytes]s to the [TransactionManager], and one to receive [TransactionReceipt]s.
    /// Spawns the [TransactionManager] in a new [std::thread::Thread].
    ///
    /// Returns a [JoinHandle] to the spawned [TransactionManager] if successfully spawed.
    pub fn spawn_transaction_manager(&mut self) -> Result<()> {
        let (tx_mgr_sender, archon_receiver) =
            mpsc::channel::<Pin<Box<TransactionReceipt>>>();
        let (archon_sender, tx_mgr_receiver) = mpsc::channel::<Pin<Box<Bytes>>>();
        self.tx_manager_sender = Some(archon_sender);
        self.tx_manager_receiver = Some(archon_receiver);
        let transaction_manager = self.tx_manager.take();
        let mut transaction_manager =
            transaction_manager.unwrap_or(TransactionManager::new(
                Some(self.config.network.into()),
                Some(self.config.batcher_inbox),
                Some(self.config.proposer_address),
                Some(self.config.batcher_private_key.clone()),
                self.config.get_l1_client()?,
            ));
        transaction_manager.with_sender(tx_mgr_sender);
        transaction_manager.with_receiver(tx_mgr_receiver);
        self.tx_manager_handle = Some(
            transaction_manager
                .spawn()
                .map_err(|_| eyre::eyre!("Failed to spawn transaction manager"))?,
        );
        Ok(())
    }

    /// Builds a new [Driver] instance.
    pub fn build_driver(&mut self) -> Result<Receiver<Pin<Box<BlockId>>>> {
        let (sender, receiver) = mpsc::channel::<Pin<Box<BlockId>>>();
        let driver = self.driver.take();
        let mut driver = if let Some(d) = driver {
            d
        } else {
            // Construct an L1 client
            let l1_client = self.config.get_l1_client()?;
            let poll_interval = self.config.polling_interval;
            Driver::new(l1_client, poll_interval, None)
        };
        driver.with_channel(sender);
        self.driver = Some(driver);
        Ok(receiver)
    }

    #[allow(clippy::type_complexity)]
    /// Builds a new [ChannelManager] instance.
    pub fn build_channel_manager(
        &mut self,
        block_recv: Option<Receiver<Pin<Box<BlockId>>>>,
    ) -> Result<(Sender<Pin<Box<BlockId>>>, Receiver<Pin<Box<Bytes>>>)> {
        let (cm_sender, archon_receiver) = mpsc::channel::<Pin<Box<Bytes>>>();
        let (archon_sender, cm_receiver) = mpsc::channel::<Pin<Box<BlockId>>>();
        let channel_manager = self.channel_manager.take();
        let mut channel_manager = channel_manager.unwrap_or_default();
        channel_manager.with_sender(cm_sender);
        channel_manager.with_receiver(cm_receiver);
        channel_manager.receive_blocks(block_recv);
        Ok((archon_sender, archon_receiver))
    }

    #[allow(clippy::type_complexity)]
    /// Builds a new [TransactionManager] instance.
    pub fn build_transaction_manager(
        &mut self,
        bytes_recv: Option<Receiver<Pin<Box<Bytes>>>>,
    ) -> Result<(
        Sender<Pin<Box<Bytes>>>,
        Receiver<Pin<Box<TransactionReceipt>>>,
    )> {
        let (archon_sender, tx_mgr_receiver) = mpsc::channel::<Pin<Box<Bytes>>>();
        let (tx_mgr_sender, archon_receiver) =
            mpsc::channel::<Pin<Box<TransactionReceipt>>>();
        self.tx_manager_sender = Some(archon_sender.clone());
        // self.tx_manager_receiver = Some(archon_receiver.clone());
        let transaction_manager = self.tx_manager.take();
        let mut transaction_manager =
            transaction_manager.unwrap_or(TransactionManager::new(
                Some(self.config.network.into()),
                Some(self.config.batcher_inbox),
                Some(self.config.proposer_address),
                Some(self.config.batcher_private_key.clone()),
                self.config.get_l1_client()?,
            ));
        transaction_manager.with_sender(tx_mgr_sender);
        transaction_manager.with_receiver(tx_mgr_receiver);
        transaction_manager.receive_bytes(bytes_recv);
        Ok((archon_sender, archon_receiver))
    }

    /// Serves [Archon] metrics.
    async fn serve_metrics(&mut self) -> Result<()> {
        match &mut self.metrics {
            Some(metrics) => metrics.serve().await,
            None => Err(eyre::eyre!("Metrics not initialized")),
        }
    }

    /// [Archon]'s Batch Submission Pipeline
    /// Builds an [Archon] pipeline and spawns all the necessary threads.
    pub async fn start(&mut self) -> Result<()> {
        tracing::info!(target: "archon", "Serving archon metrics");
        self.metrics = Some(Metrics::new());

        tracing::info!(target: "archon", "Building batch submission pipeline");
        // let block_recv = self.build_driver()?;
        // let (_, bytes_recv) = self.build_channel_manager(Some(block_recv))?;
        // let (_, receipt_recv) = self.build_transaction_manager(Some(bytes_recv))?;

        let receipt_recv = PipelineBuilder::<()>::new(self)
            .channel(Driver::default())
            .channel(ChannelManager::default())
            .channel(TransactionManager::default())
            .build();

        tracing::info!(target: "archon", "Spawning batch submission pipeline");
        self.spawn_driver()?;
        self.spawn_channel_manager()?;
        self.spawn_transaction_manager()?;

        // Receipt transactions
        let receipt_recv = receipt_recv;
        for receipt in receipt_recv {
            tracing::info!(target: "archon", "Received receipt: {:?}", receipt);
        }

        tracing::info!(target: "archon", "Serving metrics on batch submission");
        self.serve_metrics().await?;
        Ok(())
    }
}
