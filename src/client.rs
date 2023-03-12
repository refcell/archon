use std::{sync::mpsc::{self, Receiver}, pin::Pin};

use bytes::Bytes;
use tokio::task::JoinHandle;
use eyre::Result;
use ethers_core::types::BlockId;

use crate::{
	config::Config,
	driver::Driver,
    channels::ChannelManager,
	errors::ArchonError,
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
    /// The internal driver receiver
    receiver: Option<Receiver<Pin<Box<BlockId>>>>,
    /// The inner [ChannelManager]
    channel_manager: Option<ChannelManager>,
    /// A join handle on the [ChannelManager]
    channel_manager_handle: Option<JoinHandle<Result<()>>>,
    /// The internal [ChannelManager] receiver
    channel_manager_receiver: Option<Receiver<Pin<Box<Bytes>>>>,
    /// The internal [ChannelManager] sender
    channel_manager_sender: Option<Receiver<Pin<Box<BlockId>>>>,
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

    /// Instantiates a [Driver] if needed.
    /// Opens up a [std::sync::mpsc::channel] with the created [Driver].
    /// Spawns the [Driver] in a new [std::thread::Thread].
    ///
    /// Returns a [JoinHandle] to the spawned [Driver] if successfully spawed.
    pub fn spawn_driver(&mut self) -> Result<JoinHandle<Result<()>>> {
        let (sender, receiver) = mpsc::channel::<Pin<Box<BlockId>>>();
        let driver = if let Some(d) = &self.driver {
            d.with_channel(sender);
            d
        } else {
            // Construct an L1 client
            let l1_client = self.config.get_l1_client()?;
            let poll_interval = self.config.polling_interval;
            &Driver::new(l1_client, poll_interval, Some(sender))
        };
        let driver_handle = driver.spawn();
        self.receiver = Some(receiver);
        Ok(driver_handle?)
    }

    /// Instantiates a [ChannelManager] if needed.
    /// Opens up two [std::sync::mpsc::channel]s with the created [ChannelManager].
    /// One to send [BlockId]s to the [ChannelManager], and one to receive [Bytes].
    /// Spawns the [ChannelManager] in a new [std::thread::Thread].
    ///
    /// Returns a [JoinHandle] to the spawned [ChannelManager] if successfully spawed.
    pub fn spawn_channel_manager(&mut self) -> Result<JoinHandle<Result<()>>> {
        let (sender, receiver) = mpsc::channel::<Pin<Box<Bytes>>>();
        let (sender, receiver) = mpsc::channel::<Pin<Box<BlockId>>>();
        let channel_manager = if let Some(cm) = &self.channel_manager {
            d.with_sender(sender);
            d
        } else {
            // Construct an L1 client
            let l1_client = self.config.get_l1_client()?;
            let poll_interval = self.config.polling_interval;
            &Driver::new(l1_client, poll_interval, Some(sender))
        };
        let driver_handle = driver.spawn();
        self.receiver = Some(receiver);
        Ok(driver_handle?)
    }

    /// Runs the Archon pipeline
    pub async fn start(&mut self) -> Result<()> {
		// Grab the sequencer private key from the config
		let sequencer_priv_key = self.config.get_sequencer_priv_key();

		// Grab the proposer private key from the config
		let proposer_priv_key = self.config.get_proposer_priv_key();

		// Construct an L2 client
		let l2_client = self.config.get_l2_client()?;

        tracing::info!(target: "archon", "Starting batch submission pipeline");

        // Build and spawn a driver
        let driver_handle = self.spawn_driver()?;
        self.driver_handle = Some(driver_handle);

        // Build and spawn a channel manager
        let channel_manager_handle = self.spawn_channel_manager()?;
        self.channel_manager_handle = Some(channel_manager_handle);



		// TODO: Construct a rollup client
		// TODO: Use https://github.com/a16z/magi

		// TODO: Log batch submitter balance here

        // TODO: By default use the safe L2 block number from the l1 "rollup node"



        // Fetch the latest L2 block
        // let latest_l2_block = l2_client.get_block(BlockNumber::Latest).await.unwrap_or(None);
        // tracing::info!(target: "archon", "Latest L2 block: {:?}", latest_l2_block);


		// // Connect over websockets
		// let provider = Provider::new(Ws::connect("ws://localhost:8545").await?);

		// TODO: Construct a new sequencer "service" to feed into the batch driver

		// TODO: Construct a new proposer "service" to feed into the batch driver

    	Ok(())
    }
}