
use clap::Parser;
use eyre::Result;

use archon::{telemetry, config::Cli, client::Archon, batch::Batcher};

#[tokio::main]
async fn main() -> Result<()> {
    telemetry::init(false)?;
    telemetry::register_shutdown();

    // Parse CLI arguments
    let cli = Cli::parse();
    let config = cli.to_config();

    // Run batch submission
    // This will block until complete, or erroring
    let mut archon = Archon::new(Some(config));
    let batcher = Batcher::new();
    archon.with_batcher(batcher);
    archon.start().await
}
