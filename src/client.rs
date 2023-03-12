use eyre::Result;
use ethers_providers::Middleware;
use ethers_core::types::BlockNumber;

use crate::{
	config::Config,
	batch::Batcher,
	errors::ArchonError
};

/// Archon
///
/// This is the primary Archon client.
///
/// It is responsible for orchestrating the batch submission pipeline.
#[derive(Debug, Clone)]
pub struct Archon {
    /// The inner batcher
    pub batcher: Batcher,
    /// The inner config
    config: Config,
}


impl Archon {
    /// Constructs a new Archon instance from an optional [Config]
    pub fn new(config: Option<Config>) -> Self {
        Self {
            batcher: Batcher::new(),
            config: config.unwrap_or_default(),
        }
    }

	/// Sets the [Batcher] instance on the [Archon] client
    pub fn with_batcher(&mut self, batcher: Batcher) -> &mut Self {
        self.batcher = batcher;
        self
    }

    /// Runs the Archon pipeline
    pub async fn start(&mut self) -> Result<()> {
		// Grab the sequencer private key from the config
		let sequencer_priv_key = self.config.get_sequencer_priv_key();

		// Grab the proposer private key from the config
		let proposer_priv_key = self.config.get_proposer_priv_key();

		// Construct an L1 client
		let l1_client = self.config.get_l1_client()?;

		// Construct an L2 client
		let l2_client = self.config.get_l2_client()?;

		// TODO: Construct a rollup client
		// TODO: Use https://github.com/a16z/magi

		// TODO: Log batch submitter balance here

        // TODO: By default use the safe L2 block number from the l1 "rollup node"


        tracing::info!(target: "archon", "Starting batch submission pipeline");

        // Load L2 blocks into state
        self.batcher.load_l2_blocks().await?;


        // Fetch the latest L2 block
        let latest_l2_block = l2_client.get_block(BlockNumber::Latest).await.unwrap_or(None);
        tracing::info!(target: "archon", "Latest L2 block: {:?}", latest_l2_block);




        tracing::info!(target: "archon", "...");



		// // Connect over websockets
		// let provider = Provider::new(Ws::connect("ws://localhost:8545").await?);

		// TODO: Construct a new sequencer "service" to feed into the batch driver

		// TODO: Construct a new proposer "service" to feed into the batch driver

    	Ok(())
    }
}