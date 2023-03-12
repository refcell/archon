use std::{fmt::{Display, self}, collections::BTreeMap};

use bytes::Bytes;
use ethers_core::types::{Block, Transaction, H256, BlockId};
use eyre::Result;

use crate::{config::Config, driver::Batcher};



/// Channel Manager
#[derive(Debug, Default, Clone)]
pub struct ChannelManager {
    /// List of all blocks since the last request.
    blocks: Vec<Block<Transaction>>,
    /// Tip is the last block hash for reorg detection.
    tip: Option<H256>,
    /// TODO: add a channel here that is used to communicate between the driver and the channel manager
    /// An internal list of pending transactions.
    pending_txs: BTreeMap<TransactionID, Bytes>,
    /// An internal list of confirmed transactions.
    confirmed_txs: BTreeMap<TransactionID, BlockId>,
}

/// PendingChannel is a constructed pending channel
#[derive(Debug, Clone, Hash, PartialEq, PartialOrd)]
pub struct PendingChannel {

}

impl Iterator for ChannelManager {
    type Item = PendingChannel;

    fn next(&mut self) -> Option<Self::Item> {
        let batch = self.batcher.lock().ok().and_then(|mut s| s.next())?;
        self.construct_pending_channel(batch)
    }
}

impl ChannelManager {
    /// Constructs a new Channel Manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear
    ///
    /// Clears the channel manager.
    /// All of channel state is cleared.
    /// Clear is intended to be used after an L2 reorg.
    pub fn clear(&mut self) -> Result<()> {
        self.blocks.clear();
        self.tip = None;
        self.clear_pending_channels();
        Ok(())
    }

    /// resets all pending state back to an initialized but empty state.
    pub fn clear_pending_channels(&mut self) -> Result<()> {
        self.pending_txs.clear();
        self.confirmed_txs.clear();
        Ok(())
    }

    /// 
    pub fn construct_pending_channel(&self) -> Result<PendingChannel> {

    }

    /// Adds an L2 block to the internal blocks queue.
    /// It returns a [ChannelManagerError] if the block does not extend the last block loaded into the state.
    /// If no blocks were added yet, the parent hash check is skipped.
    pub fn push_l2_block(&mut self, block: Block<Transaction>) -> Result<()> {
        if self.tip.is_some() && self.tip != Some(block.parent_hash) {
            return ChannelManagerError::L1Reorg;
        }
        self.blocks.push(block);
        self.tip = Some(block.hash);
        Ok(())
    }
}


/// TransactionID is an opaque identifier for a transaction.
/// It's internal fields should not be inspected after creation & are subject to change.
/// This ID must be trivially comparable & work as a map key.
#[derive(Debug, Hash, Clone, PartialEq, PartialOrd)]
pub struct TransactionID {
    /// The channel id
    channel_id: String,
    /// The frame number
    frame_number: u64
}

impl Display for TransactionID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.channel_id, self.frame_number)
    }
}

/// TaggedData tags raw byte data with an associated [TransactionID]
pub struct TaggedData {
    /// The internal data
    data: Bytes,
    /// The associated transaction id
    id: TransactionID,
}