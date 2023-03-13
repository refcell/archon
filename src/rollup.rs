//! Rollup
//!
//! Encapsulates logic for interacting with a rollup node.

use ethers_providers::{Http, Provider};
use eyre::Result;

/// A Rollup Node
#[derive(Debug, Clone, Default)]
pub struct RollupNode {
    /// The rollup node's URL.
    pub l2_client: Option<Provider<Http>>,
}

impl RollupNode {
    /// Creates a new rollup node.
    pub fn new(l2_url: &str) -> Result<Self> {
        let l2_client = Provider::<Http>::try_from(l2_url)?;
        Ok(Self {
            l2_client: Some(l2_client),
        })
    }

    /// Returns the sync status of the rollup node as a [`SyncStatus`].
    ///
    /// This should be called synchronously with the driver event loop
    /// to avoid retrieval of an inconsistent status.
    pub async fn sync_status(&self) -> Result<SyncStatus> {
        // TODO: Implement
        tracing::warn!(target: "archon::rollup", "sync_status() not implemented yet on the rollup node");
        Ok(SyncStatus::default())
    }
}

/// The current sync status of a rollup node.
#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SyncStatus {
    /// The current L1 block number.
    pub current_l1: u64,
    /// The current L1 finalized block number.
    pub current_l1_finalized: u64,
    /// The current L1 head block number.
    pub head_l1: u64,
    /// The current L1 safe block number.
    pub safe_l1: u64,
    /// The current L1 finalized block number.
    pub finalized_l1: u64,
    /// The current L2 head block number.
    pub unsafe_l2: u64,
    /// The current L2 safe block number.
    pub safe_l2: u64,
    /// The current L2 finalized block number.
    pub finalized_l2: u64,
}
