use eyre::Result;
use crate::config::Config;


/// Batcher
///
/// Encapsulates batch submission logic.
#[derive(Debug, Default, Clone)]
pub struct Batcher {
    /// The chain ID
    chain_id: u64,
    /// The data directory
    data_dir: String,
}

impl Batcher {
    /// Constructs a new Batcher instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Batch submission pipeline
    pub async fn batch_submission(&self, config: &Config) -> Result<()> {
        println!("Inside batch submission pipeline!");
        Ok(())
    }

    /// Load L2 Blocks into state
    pub async fn load_l2_blocks(&self) -> Result<()> {
        tracing::error!(target: "archon", "Inside load L2 blocks!");


        // loadBlocksIntoState loads all blocks since the previous stored block
        // It does the following:
        // 1. Fetch the sync status of the sequencer
        // 2. Check if the sync status is valid or if we are all the way up to date
        // 3. Check if it needs to initialize state OR it is lagging (todo: lagging just means race condition?)
        // 4. Load all new blocks into the local state.



        // start, end, err := l.calculateL2BlockRangeToStore(ctx)
        // if err != nil {
        //     l.log.Trace("was not able to calculate L2 block range", "err", err)
        //     return
        // }

        // // Add all blocks to "state"
        // for i := start.Number + 1; i < end.Number+1; i++ {
        //     id, err := l.loadBlockIntoState(ctx, i)
        //     if errors.Is(err, ErrReorg) {
        //         l.log.Warn("Found L2 reorg", "block_number", i)
        //         l.state.Clear()
        //         l.lastStoredBlock = eth.BlockID{}
        //         return
        //     } else if err != nil {
        //         l.log.Warn("failed to load block into state", "err", err)
        //         return
        //     }
        //     l.lastStoredBlock = id
        // }

        Ok(())
    }
}
