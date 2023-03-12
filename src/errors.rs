

use thiserror::Error;

/// Archon Error
#[derive(Debug, Error)]
pub enum ArchonError {
    /// Missing Batcher
    #[error("missing batcher")]
    MissingBatcher,
}

/// Configuration Error
#[derive(Debug, Error)]
pub enum ConfigError {
    /// L1 Client URL is invalid
    #[error("l1 client url is invalid")]
    InvalidL1ClientUrl,
    /// L2 Client URL is invalid
    #[error("l2 client url is invalid")]
    InvalidL2ClientUrl,
}


/// ChannelManager Error
#[derive(Debug, Error)]
pub enum ChannelManagerError {
    /// L1 reorg
    #[error("l1 reorg")]
    L1Reorg,
    /// L2 reorg
    #[error("l2 reorg")]
    L2Reorg,
}
