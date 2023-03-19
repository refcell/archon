use ethers_core::types::{
    Block,
    Transaction,
    H256,
};
use serde::{
    Deserialize,
    Serialize,
};

/// A block update.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BlockUpdate {
    /// The block was added to the chain
    Added,
    /// A reorg occurred
    Reorg,
    /// Block is missing a hash
    MissingBlockHash,
}

/// [State] handles the processing of L2 blocks.
///
/// It drives the inner workings of the [crate::channels::ChannelManager].
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct State {
    /// An internal block store
    blocks: Vec<Block<Transaction>>,
    /// Tracks the current block tip
    tip: Option<H256>,
}

impl State {
    /// Constructs a new [State] instance.
    pub fn new() -> Self {
        Self { ..Self::default() }
    }

    /// Adds an L2 Block to [State].
    /// It returns a [BlockUpdate::Reorg] if the block does not extend the last block loaded into the state.
    /// If no blocks were added yet, the parent hash check is skipped.
    pub fn add_block(&mut self, block: Block<Transaction>) -> BlockUpdate {
        if self.tip.is_some() && self.tip != Some(block.parent_hash) {
            return BlockUpdate::Reorg
        }
        match block.hash {
            Some(h) => self.tip = Some(h),
            None => return BlockUpdate::MissingBlockHash,
        }
        self.blocks.push(block);
        BlockUpdate::Added
    }

    /// Clears the [State] of all blocks and pending channels.
    pub fn clear(&mut self) {
        self.blocks.clear();
        self.tip = None;
    }
}
