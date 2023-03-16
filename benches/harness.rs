use archon::client::*;
use ethers_core::types::{BlockId, BlockNumber, U64};
use eyre::Result;

/// Blocks a new [tokio::runtime::Runtime] and runs the given future.
///
/// h/t @ https://github.com/smrpn
/// rev: https://github.com/smrpn/casbin-rs/commit/7a0a75d8075440ee65acdac3ee9c0de6fcbd5c48
pub fn await_future<F: std::future::Future<Output = T>, T>(future: F) -> T {
    tokio::runtime::Runtime::new().unwrap().block_on(future)
}

/// Constructs a new [Archon] client with mock channels.
#[allow(dead_code)]
pub fn mock_archon_client() -> Result<Archon> {
    Ok(Archon::new(None))
}

/// Returns a mock [BlockId] with a [BlockNumber] of 100.
pub async fn fetch_latest_block_id() -> Result<BlockId> {
    Ok(BlockId::Number(BlockNumber::Number(U64::from(100))))
}

/// Create a tokio multi-threaded [tokio::runtime::Runtime].
///
/// # Panics
///
/// Panics if the runtime cannot be created.
pub fn construct_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
}
