use std::sync::{Arc, Mutex};
use std::{pin::Pin, sync::mpsc::Sender, time::Duration};

use ethers_core::types::{BlockId, BlockNumber};
use ethers_providers::Middleware;
use ethers_providers::{Http, Provider};
use eyre::Result;

/// Driver handles the driving of the batch submission pipeline.
#[derive(Debug, Default, Clone)]
pub struct Driver {
    /// Polling interval - interval to poll L1 blocks at
    poll_interval: Duration,
    /// The provider
    provider: Option<Provider<Http>>,
    /// A channel to send messages back to the spawner
    sender: Option<Sender<Pin<Box<BlockId>>>>,
}

impl Driver {
    /// Constructs a new Driver instance
    pub fn new(
        provider: Provider<Http>,
        poll_interval: Option<Duration>,
        sender: Option<Sender<Pin<Box<BlockId>>>>,
    ) -> Self {
        Self {
            provider: Some(provider),
            poll_interval: poll_interval.unwrap_or(Duration::from_secs(5)),
            sender,
        }
    }

    /// Sets the [Driver] [Sender] channel.
    ///
    /// Returns a mutable reference to the [Driver] instance.
    pub fn with_channel(&mut self, sender: Sender<Pin<Box<BlockId>>>) -> &mut Self {
        self.sender = Some(sender);
        self
    }

    /// Spawns the [Driver] into a new thread
    pub fn spawn(self) -> Result<tokio::task::JoinHandle<Result<()>>> {
        let provider = self
            .provider
            .clone()
            .ok_or(eyre::eyre!("Driver missing provider!"))?;
        let sender = self.sender.ok_or(eyre::eyre!("Driver missing sender!"))?;
        let sender = Arc::new(Mutex::new(sender));
        let interval = self.poll_interval;
        let driver_handle = tokio::spawn(async move {
            tracing::info!(target: "archon::driver", "Spawning driver in new thread...");
            Driver::execute(interval, sender, provider).await
        });
        Ok(driver_handle)
    }

    /// Executes the driver
    pub async fn execute(
        interval: Duration,
        sender: Arc<Mutex<Sender<Pin<Box<BlockId>>>>>,
        provider: Provider<Http>,
    ) -> Result<()> {
        tracing::info!(target: "archon::driver", "Executing driver...");
        let mut first_iter = true;
        loop {
            // Await the poll interval at the loop start so we can ergonomically continue below.
            if !first_iter {
                std::thread::sleep(interval);
            }
            first_iter = false;

            // Read the latest l1 block from the provider.
            let l1_tip = match provider.get_block(BlockId::Number(BlockNumber::Latest)).await {
                Ok(Some(t)) => t,
                Ok(None) => {
                    tracing::warn!(target: "archon::driver", "failed to fetch latest l1 block, got None!");
                    continue;
                }
                Err(e) => {
                    tracing::warn!(target: "archon::driver", "failed to fetch latest l1 block!\nError: {}", e);
                    continue;
                }
            };
            tracing::info!(target: "archon::driver", "Fetched latest l1 block");

            // Derive a [BlockId] from the fetched [Block].
            let block_id = if let Some(h) = l1_tip.hash {
                BlockId::from(h)
            } else if let Some(n) = l1_tip.number {
                BlockId::from(n)
            } else {
                tracing::warn!(target: "archon::driver", "block response missing both number and hash, failed to construct block id!");
                continue;
            };
            tracing::info!(target: "archon::driver", "Latest L1 block id: {:?}", block_id);

            // Pass back the latest L1 block id to the spawner.
            // We lock here and not across the loop to prevent deadlocking other threads.
            let locked = if let Ok(s) = sender.lock() {
                s
            } else {
                continue;
            };
            if let Err(e) = locked.send(Box::pin(block_id)) {
                tracing::warn!(target: "archon::driver", "failed to send block id {:?} to spawner: {}", block_id, e);
            }
        }
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
