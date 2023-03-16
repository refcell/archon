use clap::Parser;
use eyre::Result;

use archon::{client::Archon, config::Cli, telemetry};

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
    match archon.start().await {
        Ok(_) => Ok(()),
        Err(e) => {
            tracing::error!(target: "archon", "Archon exited with error: {}", e);
            Err(e)
        }
    }
}
