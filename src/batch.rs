

use eyre::Result;
use crate::config::Config;


/// Batcher
///
/// Encapsulates batch submission logic.
#[derive(Debug, Default, Clone)]
pub struct Batcher {
    /// The chain ID
    chain_id: u64,
    /// The data directory
    data_dir: String,
}

impl Batcher {
    /// Constructs a new Batcher instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Batch submission pipeline
    pub async fn batch_submission(&self, config: &Config) -> Result<()> {
        println!("Inside batch submission pipeline!");
        Ok(())
    }
}
