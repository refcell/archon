use std::{
    collections::BTreeMap,
    fmt::{self, Display},
    pin::Pin,
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
};

use bytes::Bytes;
use ethers_core::types::{Block, BlockId, Transaction, H256};
use eyre::Result;

use crate::errors::ChannelManagerError;

/// Channel Manager
#[derive(Debug, Default)]
pub struct ChannelManager {
    /// List of all blocks since the last request.
    blocks: Vec<Block<Transaction>>,
    /// Tip is the last block hash for reorg detection.
    tip: Option<H256>,
    /// A channel to send [Bytes] back to the [crate::client::Archon] orchestrator
    sender: Option<Sender<Pin<Box<Bytes>>>>,
    /// A channel to receive [BlockId] messages from the [crate::client::Archon] orchestrator
    receiver: Option<Receiver<Pin<Box<BlockId>>>>,
    /// An internal map of pending transactions.
    pending_txs: BTreeMap<TransactionID, Bytes>,
    /// An internal map of confirmed transactions.
    confirmed_txs: BTreeMap<TransactionID, BlockId>,
    /// A block receiver
    block_recv: Option<Receiver<Pin<Box<BlockId>>>>,
}

/// PendingChannel is a constructed pending channel
#[derive(Debug, Clone, Hash, PartialEq, PartialOrd)]
pub struct PendingChannel {}

// impl Iterator for ChannelManager {
//     type Item = PendingChannel;

//     fn next(&mut self) -> Option<Self::Item> {
//         let batch = self.batcher.lock().ok().and_then(|mut s| s.next())?;
//         self.construct_pending_channel(batch)
//     }
// }

impl ChannelManager {
    /// Constructs a new Channel Manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the [ChannelManager] sender.
    ///
    /// This [std::sync::mpsc::channel] is used to send [Bytes] back to the [crate::client::Archon] orchestrator.
    pub fn with_sender(&mut self, sender: Sender<Pin<Box<Bytes>>>) -> &mut Self {
        self.sender = Some(sender);
        self
    }

    /// Sets the [ChannelManager] receiver.
    ///
    /// This [std::sync::mpsc::channel] is used by the [crate::client::Archon] orchestrator to send
    /// [BlockId] messages to the [ChannelManager]. [BlockId]s sent through this channel are expected
    /// to be the latest L1 [BlockId] fetched via a [ethers_providers::Provider].
    ///
    /// Optionally, the [ChannelManager] should validate that the [BlockId] is the valid latest L1 [BlockId].
    pub fn with_receiver(&mut self, receiver: Receiver<Pin<Box<BlockId>>>) -> &mut Self {
        self.receiver = Some(receiver);
        self
    }

    /// Sets the [ChannelManager] receiever
    pub fn receive_blocks(&mut self, block_recv: Option<Receiver<Pin<Box<BlockId>>>>) -> &mut Self {
        self.block_recv = block_recv;
        self
    }

    /// Constructs the next transaction data that should be submitted to L1.
    ///
    /// Transaction data is returned as raw [Bytes].
    /// It currently only uses one frame per transaction. If the pending channel is
    /// full, it only returns the remaining frames of this channel until it got
    /// successfully fully sent to L1. It returns an error if there's no pending frame.
    pub fn tx_data(block_id: BlockId) -> Result<(Bytes, TransactionID)> {
        tracing::debug!(target: "archon::channels", "channel manager constructing tx data with block id: {:?}...", block_id);
        // TODO: implement
        Err(ChannelManagerError::NotImplemented.into())
    }

    /// Executes the [ChannelManager].
    pub async fn execute(
        receiver: Arc<Mutex<Receiver<Pin<Box<BlockId>>>>>,
        sender: Arc<Mutex<Sender<Pin<Box<Bytes>>>>>,
    ) -> Result<()> {
        // TODO: pull pending transactions up to the [ChannelManager] state
        let mut pending_txs = BTreeMap::new();
        loop {
            let locked_receiver = receiver
                .lock()
                .map_err(|_| ChannelManagerError::ReceiverLock)?;
            let block_id = locked_receiver
                .recv()
                .map_err(|_| ChannelManagerError::ChannelClosed)?;
            let (tx_data, tx_id) = Self::tx_data(*block_id)?;
            let locked_sender = sender.lock().map_err(|_| ChannelManagerError::SenderLock)?;
            locked_sender.send(Box::pin(tx_data.clone()))?;
            pending_txs.insert(tx_id, tx_data);
        }
    }

    /// Spawns the [ChannelManager] into a new thread
    pub fn spawn(self) -> Result<tokio::task::JoinHandle<Result<()>>> {
        let receiver = self
            .receiver
            .ok_or(eyre::eyre!("ChannelManager missing receiver!"))?;
        let receiver = Arc::new(Mutex::new(receiver));
        let sender = self
            .sender
            .ok_or(eyre::eyre!("ChannelManager missing sender!"))?;
        let sender = Arc::new(Mutex::new(sender));
        let channel_manager_handle = tokio::spawn(async move {
            tracing::info!(target: "archon::channels", "Spawned ChannelManager in a new thread");
            ChannelManager::execute(receiver, sender).await
        });
        Ok(channel_manager_handle)
    }

    /// Clear
    ///
    /// Clears the channel manager.
    /// All of channel state is cleared.
    /// Clear is intended to be used after an L2 reorg.
    pub fn clear(&mut self) -> Result<()> {
        self.blocks.clear();
        self.tip = None;
        self.clear_pending_channels()?;
        Ok(())
    }

    /// resets all pending state back to an initialized but empty state.
    pub fn clear_pending_channels(&mut self) -> Result<()> {
        self.pending_txs.clear();
        self.confirmed_txs.clear();
        Ok(())
    }

    /// Constructs a [PendingChannel].
    pub fn construct_pending_channel(&self) -> Result<PendingChannel> {
        Err(ChannelManagerError::NotImplemented.into())
    }

    /// Adds an L2 block to the internal blocks queue.
    /// It returns a [ChannelManagerError] if the block does not extend the last block loaded into the state.
    /// If no blocks were added yet, the parent hash check is skipped.
    pub fn push_l2_block(&mut self, block: Block<Transaction>) -> Result<()> {
        if self.tip.is_some() && self.tip != Some(block.parent_hash) {
            return Err(ChannelManagerError::L1Reorg.into());
        }
        self.tip = block.hash;
        self.blocks.push(block);
        Ok(())
    }
}

/// TransactionID is an opaque identifier for a transaction.
/// It's internal fields should not be inspected after creation & are subject to change.
/// This ID must be trivially comparable & work as a map key.
#[derive(Debug, Hash, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct TransactionID {
    /// The channel id
    channel_id: String,
    /// The frame number
    frame_number: u64,
}

impl Default for TransactionID {
    fn default() -> Self {
        Self {
            channel_id: String::from("0:0"),
            frame_number: 0,
        }
    }
}

impl Display for TransactionID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.channel_id, self.frame_number)
    }
}

/// TaggedData tags raw byte data with an associated [TransactionID]
#[derive(Debug, Hash, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct TaggedData {
    /// The internal data
    data: Bytes,
    /// The associated transaction id
    id: TransactionID,
}
