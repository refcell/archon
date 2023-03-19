use eyre::Result;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

/// Metrics
///
/// Serves metrics for the [crate::client::Archon] client.
#[derive(Debug, Default, Clone)]
pub struct Metrics {}

impl Metrics {
    /// Constructs a new [Metrics] instance
    pub fn new() -> Self {
        Self {}
    }

    /// Serve a [TcpListener] to provide [crate::client::Archon] metrics.
    pub async fn serve(&mut self) -> Result<()> {
        let addr = "127.0.0.1:8082".to_string();
        let listener = TcpListener::bind(&addr)
            .map_err(|_| eyre::eyre!("Metrics failed to bind to {}", addr))?;
        for stream in listener.incoming().flatten() {
            self.handle_connection(stream)?;
        }
        Ok(())
    }

    /// Handle an incoming connection.
    pub fn handle_connection(&self, mut stream: TcpStream) -> Result<()> {
        let mut buffer = [0; 1024];
        let read_bytes = stream.read(&mut buffer)?;
        println!(
            "Request with {} bytes: {}",
            read_bytes,
            String::from_utf8_lossy(&buffer[..])
        );
        Ok(())
    }
}
