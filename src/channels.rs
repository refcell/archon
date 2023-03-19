use std::{
    collections::BTreeMap,
    fmt::{
        self,
        Display,
    },
    pin::Pin,
    sync::{
        mpsc::{
            Receiver,
            Sender,
        },
        Arc,
        Mutex,
    },
    time::Duration,
};

use bytes::Bytes;
use ethers_core::types::BlockId;
use ethers_providers::{
    Http,
    Middleware,
    Provider,
};
use eyre::Result;
use tokio::task::JoinHandle;

use crate::{
    errors::ChannelManagerError,
    rollup::RollupNode,
    state::State,
};

/// Channel Manager
#[derive(Debug, Default)]
pub struct ChannelManager {
    /// Internal [State] Manager
    state: Arc<Mutex<State>>,
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
    pub fn receive_blocks(
        &mut self,
        block_recv: Option<Receiver<Pin<Box<BlockId>>>>,
    ) -> &mut Self {
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
        block_recv: Option<Receiver<Pin<Box<BlockId>>>>,
        receiver: Arc<Mutex<Receiver<Pin<Box<BlockId>>>>>,
        sender: Arc<Mutex<Sender<Pin<Box<Bytes>>>>>,
    ) -> Result<()> {
        let mut pending_txs = BTreeMap::new();
        loop {
            // Read block id from the receiver.
            // This will block until a new block id is received.
            let block_id = if let Some(block_recv) = &block_recv {
                block_recv
                    .recv()
                    .map_err(|_| ChannelManagerError::ChannelClosed)?
            } else {
                let locked_receiver = receiver
                    .lock()
                    .map_err(|_| ChannelManagerError::ReceiverLock)?;
                locked_receiver
                    .recv()
                    .map_err(|_| ChannelManagerError::ChannelClosed)?
            };
            let (tx_data, tx_id) = Self::tx_data(*block_id)?;
            let locked_sender =
                sender.lock().map_err(|_| ChannelManagerError::SenderLock)?;
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
            ChannelManager::execute(self.block_recv, receiver, sender).await
        });
        Ok(channel_manager_handle)
    }

    /// Spawns a separate thread to process L2 blocks.
    pub fn spawn_block_processor(
        &mut self,
        rollup_node_rpc_url: &str,
        l2_node_rpc_url: &str,
        interval: Duration,
    ) -> Result<JoinHandle<Result<()>>> {
        let rollup_node = RollupNode::new(rollup_node_rpc_url)?;
        let l2_rpc = Provider::<Http>::try_from(l2_node_rpc_url)?;
        let state = self.state.clone();

        // Spawn the block processor in a separate thread.
        let channel_manager_handle = tokio::spawn(async move {
            tracing::info!(target: "archon::channels", "Spawned ChannelManager in a new thread");
            ChannelManager::process_blocks(rollup_node, l2_rpc, interval, state).await
        });
        Ok(channel_manager_handle)
    }

    /// Handles the processing of L2 blocks.
    pub async fn process_blocks(
        rollup_node: RollupNode,
        l2_node: Provider<Http>,
        polling_interval: Duration,
        state: Arc<Mutex<State>>,
    ) -> Result<()> {
        tracing::info!(target: "archon::channels", "Executing block processor...");
        let mut first_iter = true;
        let mut last_stored_block_number = 0;
        loop {
            // Await the poll interval at the loop start so we can ergonomically continue below.
            if !first_iter {
                std::thread::sleep(polling_interval);
            }
            first_iter = false;

            // Calculate the range of L2 blocks to process
            let (start_block, end_block) = {
                let sync_status = match rollup_node.sync_status().await {
                    Ok(sync_status) => sync_status,
                    Err(err) => {
                        tracing::error!(target: "archon::channels", "Failed to fetch rollup node sync status: {:?}", err);
                        continue
                    }
                };
                if sync_status.head_l1 == 0 {
                    tracing::warn!(target: "archon::channels", "Rollup node is not synced yet. Waiting for rollup node to sync...");
                    continue
                }
                if last_stored_block_number == 0
                    || last_stored_block_number < sync_status.safe_l2
                {
                    last_stored_block_number = sync_status.safe_l2;
                }
                (last_stored_block_number, sync_status.unsafe_l2)
            };

            // Process the L2 blocks
            for block_number in (start_block + 1)..=(end_block + 1) {
                let block = match l2_node.get_block_with_txs(block_number).await {
                    Ok(Some(block)) => block,
                    _ => {
                        tracing::error!(target: "archon::channels", "Failed to fetch L2 block");
                        continue
                    }
                };
                match state.lock() {
                    Ok(mut s) => match block.number {
                        Some(num) => {
                            last_stored_block_number = num.as_u64();
                            s.add_block(block);
                        }
                        None => {
                            tracing::error!(target: "archon::channels", "Failed to fetch L2 block number");
                            continue
                        }
                    },
                    Err(_) => {
                        tracing::error!(target: "archon::channels", "Failed to lock state");
                        continue
                    }
                }
                tracing::debug!(target: "archon::channels", "Processed L2 block: {:?}", last_stored_block_number);
            }
        }
    }

    /// Clear
    ///
    /// Clears the channel manager.
    /// All of channel state is cleared.
    /// Clear is intended to be used after an L2 reorg.
    pub fn clear(&mut self) -> Result<()> {
        self.state
            .lock()
            .map_err(|_| eyre::eyre!("Failed to lock state to clear"))?
            .clear();
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
