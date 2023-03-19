use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use eyre::Result;

/// Metrics
///
/// Serves metrics for the [Archon] client.
#[derive(Debug, Default, Clone)]
pub struct Metrics {}

impl Metrics {
    /// Constructs a new [metrics] instance
    pub fn new() -> Self {
        Self { }
    }

    /// Spawn an http server to provide [Archon] metrics.
    pub async fn spawn(&mut self) -> Result<()> {
        let addr = format!("127.0.0.1:8082");
        let listener = TcpListener::bind(&addr).map_err(|_| eyre::eyre!("Metrics failed to bind to {}", addr))?;
        for stream in listener.incoming() {
            if let Ok(stream) = stream {
                self.handle_connection(stream);
            }
        }
        Ok(())
    }

    // TODO: Properly handle incoming connections.
    // TODO: Is there an out-of-the-box solution for serving metrics?
    /// Handle an incoming connection.
    pub fn handle_connection(&self, mut stream: TcpStream) {
        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();
        println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
    }
}
