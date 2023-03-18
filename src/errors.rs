use thiserror::Error;

/// [Archon] Error
#[derive(Debug, Error)]
pub enum ArchonError {
    /// Missing Batcher
    #[error("missing batcher")]
    MissingBatcher,
}

/// [Config] Error
#[derive(Debug, Error)]
pub enum ConfigError {
    /// L1 Client URL is invalid
    #[error("l1 client url is invalid")]
    InvalidL1ClientUrl,
    /// L2 Client URL is invalid
    #[error("l2 client url is invalid")]
    InvalidL2ClientUrl,
}

/// [ChannelManager] Error
#[derive(Debug, Error)]
pub enum ChannelManagerError {
    /// L1 reorg
    #[error("l1 reorg")]
    L1Reorg,
    /// L2 reorg
    #[error("l2 reorg")]
    L2Reorg,
    /// Not Implemented
    #[error("method not implemented")]
    NotImplemented,
    /// Channel Closed
    #[error("channel closed")]
    ChannelClosed,
    /// Channel Manager failed to lock the receiver
    #[error("failed to lock the receiver")]
    ReceiverLock,
    /// Channel Manager failed to lock the sender
    #[error("failed to lock the sender")]
    SenderLock,
}

/// [TransactionManager] Error
#[derive(Debug, Error)]
pub enum TransactionManagerError {
    /// Channel Closed
    #[error("channel closed")]
    ChannelClosed,
    /// Channel Manager failed to lock the receiver
    #[error("failed to lock the receiver")]
    ReceiverLock,
    /// Channel Manager failed to lock the sender
    #[error("failed to lock the sender")]
    SenderLock,
    /// Missing Receiver Channel
    #[error("missing receiver channel")]
    MissingReceiver,
    /// Missing Sender Channel
    #[error("missing sender channel")]
    MissingSender,
    /// This error is fired when the [TransactionManager] `send_transaction`
    /// method is called concurrently.
    #[error("transaction manager sending is locked")]
    SendTransactionLocked,
    /// Missing provider
    #[error("missing provider")]
    MissingProvider,
    /// Missing sender address
    #[error("missing sender address")]
    MissingSenderAddress,
    /// Missing L1 chain ID
    #[error("missing l1 chain id")]
    MissingL1ChainId,
    /// Missing L1 batch inbox address
    #[error("missing l1 batch inbox address")]
    MissingL1BatchInboxAddress,
    /// Missing transaction receipt
    #[error("missing transaction receipt")]
    TransactionReceiptNotFound,
    /// Missing sender private key
    #[error("missing sender private key")]
    MissingSenderPrivateKey,
}
