//! Rollup
//!
//! Encapsulates logic for interacting with a rollup node.

use ethers_core::types::{BlockId, H256};
use ethers_providers::{Http, Provider};
use eyre::Result;
use serde::{Deserialize, Serialize};

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

    /// Fetches the output of the rollup node as a [`OutputResponse`].
    pub async fn output_at_block(&self, block_num: u64) -> Result<OutputResponse> {
        let output = self
            .l2_client
            .as_ref()
            .unwrap()
            .request("optimism_outputAtBlock", vec![block_num])
            .await?;
        Ok(output)
    }

    /// Fetches the sync status of the rollup node as a [`SyncStatus`].
    ///
    /// This should be called synchronously with the driver event loop
    /// to avoid retrieval of an inconsistent status.
    pub async fn sync_status(&self) -> Result<SyncStatus> {
        let empty_params: Vec<String> = Vec::new();
        let sync_status = self
            .l2_client
            .as_ref()
            .unwrap()
            .request("optimism_syncStatus", empty_params)
            .await?;
        Ok(sync_status)
    }

    /// Fetches the rollup-node's config as a [`serde_json::Value`].
    pub async fn rollup_config(&self) -> Result<serde_json::Value> {
        let empty_params: Vec<String> = Vec::new();
        let config = self
            .l2_client
            .as_ref()
            .unwrap()
            .request("optimism_rollupConfig", empty_params)
            .await?;
        Ok(config)
    }

    /// Fetches the rollup-node's version as a [`String`].
    pub async fn version(&self) -> Result<String> {
        let empty_params: Vec<String> = Vec::new();
        let version = self
            .l2_client
            .as_ref()
            .unwrap()
            .request("optimism_version", empty_params)
            .await?;
        Ok(version)
    }
}

/// The current sync status of a rollup node.
#[derive(
    Debug, Clone, Serialize, Deserialize, Default, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
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

/// The rollup node's OutputResponse.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OutputResponse {
    /// The output response version
    pub version: Vec<u8>,
    /// The output response output root
    pub output_root: Vec<u8>,
    /// The output response block ref
    pub block_ref: L2BlockRef,
    /// The output response withdrawal storage root
    pub withdrawal_storage_root: H256,
    /// The output response state root
    pub state_root: H256,
    /// The output response sync status
    pub sync_status: SyncStatus,
}

/// The rollup node's L2BlockRef.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct L2BlockRef {
    /// The L2 block ref hash
    pub hash: H256,
    /// The L2 block ref number
    pub number: u64,
    /// The L2 block ref parent hash
    pub parent_hash: H256,
    /// The L2 block ref time
    pub time: u64,
    /// The L2 block ref L1 origin
    #[serde(rename = "l1origin")]
    pub l1_origin: BlockId,
    /// The L2 block ref sequence number
    pub sequence_number: u64,
}
