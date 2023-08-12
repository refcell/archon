use crate::{
    client::Archon,
    pipeline_builder::Stage,
};
use bytes::Bytes;
use ethers_core::types::{
    Address,
    TransactionReceipt,
    TransactionRequest,
};
use ethers_middleware::SignerMiddleware;
use ethers_providers::{
    Http,
    Middleware,
    Provider,
};
use ethers_signers::LocalWallet;
use eyre::Result;
// use once_cell::sync::Lazy;
use std::{
    convert::TryFrom,
    pin::Pin,
    sync::mpsc::{
        channel,
        Receiver,
        Sender,
    },
};

use crate::errors::TransactionManagerError;

/// A global lock to prevent the [TransactionManager::send_transaction] from being called concurrently.
// static TRANSACTION_MANAGER_LOCK: Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(0));

/// Transaction Manager
#[derive(Debug, Default)]
pub struct TransactionManager {
    /// The L1 Chain ID
    l1_chain_id: Option<u64>,
    /// The batch inbox address on L1 to send transactions to
    l1_batch_inbox_address: Option<Address>,
    /// The address to send transactions from
    sender_address: Option<Address>,
    /// The private key to sign transactions with
    sender_private_key: Option<String>,
    /// The [ethers_providers::Provider] to use to send transactions
    provider: Option<Provider<Http>>,
    /// A channel to send transaction [Receipt]s back to the [crate::client::Archon] orchestrator
    sender: Option<Sender<Pin<Box<TransactionReceipt>>>>,
    /// A channel to receive [Bytes] from the [crate::client::Archon] orchestrator
    receiver: Option<Receiver<Pin<Box<Bytes>>>>,
    /// A bytes receiver
    bytes_receiver: Option<Receiver<Pin<Box<Bytes>>>>,
}

impl TransactionManager {
    /// Constructs a new [TransactionManager]
    pub fn new(
        l1_chain_id: Option<u64>,
        l1_batch_inbox_address: Option<Address>,
        sender_address: Option<Address>,
        sender_private_key: Option<String>,
        provider: Provider<Http>,
    ) -> Self {
        Self {
            l1_chain_id,
            l1_batch_inbox_address,
            sender_address,
            sender_private_key,
            provider: Some(provider),
            ..Self::default()
        }
    }

    /// Sets the [TransactionManager] sender.
    ///
    /// This [std::sync::mpsc::channel] is used to send [Receipt]s back to the [crate::client::Archon] orchestrator.
    pub fn with_sender(
        &mut self,
        sender: Sender<Pin<Box<TransactionReceipt>>>,
    ) -> &mut Self {
        self.sender = Some(sender);
        self
    }

    /// Sets the [TransactionManager] bytes receiver.
    pub fn receive_bytes(
        &mut self,
        bytes_recv: Option<Receiver<Pin<Box<Bytes>>>>,
    ) -> &mut Self {
        self.bytes_receiver = bytes_recv;
        self
    }

    /// Sets the [TransactionManager] receiver.
    ///
    /// This [std::sync::mpsc::channel] is used by the [crate::client::Archon] orchestrator to send
    /// [Bytes] messages to the [TransactionManager]. [Bytes] sent through this channel are expected
    /// to be the constructed transaction data that should be submitted to L1 built by the [crate::channels::ChannelManager].
    pub fn with_receiver(&mut self, receiver: Receiver<Pin<Box<Bytes>>>) -> &mut Self {
        self.receiver = Some(receiver);
        self
    }

    /// Sets the [TransactionManager] receiver channel.
    ///
    /// Only sets the receiver channel on the [TransactionManager]
    /// if the provided receiver is `Some`.
    pub fn receive(&mut self, bytes_recv: Option<Receiver<Pin<Box<Bytes>>>>) {
        if let Some(recv) = bytes_recv {
            self.receiver = Some(recv);
        }
    }

    #[allow(clippy::too_many_arguments)]
    /// Executes the [TransactionManager].
    pub async fn execute(
        bytes_receiver: Option<Receiver<Pin<Box<Bytes>>>>,
        l1_chain_id: u64,
        l1_batch_inbox_address: Address,
        sender_address: Address,
        _sender_private_key: String,
        provider: Provider<Http>,
        receiver: Receiver<Pin<Box<Bytes>>>,
        sender: Sender<Pin<Box<TransactionReceipt>>>,
    ) -> Result<()> {
        // TODO: construct the local wallet from a private key
        let wallet = LocalWallet::new(&mut rand::thread_rng());
        loop {
            // Receive the transaction bytes from the channel
            let tx_bytes = match &bytes_receiver {
                Some(bytes_receiver) => bytes_receiver
                    .recv()
                    .map_err(|_| TransactionManagerError::ChannelClosed)?,
                None => receiver
                    .recv()
                    .map_err(|_| TransactionManagerError::ChannelClosed)?,
            };
            let tx_bytes = tx_bytes.to_vec();
            let tx_bytes = Bytes::try_from(tx_bytes)?;

            // Build the transaction from the bytes
            let built_transaction = if let Ok(tr) = TransactionManager::craft_transaction(
                l1_chain_id,
                l1_batch_inbox_address,
                sender_address,
                &provider,
                tx_bytes,
            )
            .await
            {
                tr
            } else {
                tracing::error!(target: "archon::transactions", "Failed to craft transaction");
                continue
            };

            // Send the transaction to L1
            let tx_receipt = TransactionManager::send_transaction(
                provider.clone(),
                wallet.clone(),
                built_transaction,
            )
            .await?;
            sender.send(Box::pin(tx_receipt))?;
        }
    }

    /// Spawns the [TransactionManager] into a new thread
    pub fn spawn(self) -> Result<tokio::task::JoinHandle<Result<()>>> {
        let receiver = self
            .receiver
            .ok_or(TransactionManagerError::MissingReceiver)?;
        // let receiver = Arc::new(Mutex::new(receiver));
        let sender = self.sender.ok_or(TransactionManagerError::MissingSender)?;
        // let sender = Arc::new(Mutex::new(sender));
        let provider = self
            .provider
            .ok_or(TransactionManagerError::MissingProvider)?;
        let l1_chain_id = self
            .l1_chain_id
            .ok_or(TransactionManagerError::MissingL1ChainId)?;
        let l1_batch_inbox_address = self
            .l1_batch_inbox_address
            .ok_or(TransactionManagerError::MissingL1BatchInboxAddress)?;
        let sender_address = self
            .sender_address
            .ok_or(TransactionManagerError::MissingSenderAddress)?;
        let private_key = self
            .sender_private_key
            .ok_or(TransactionManagerError::MissingSenderPrivateKey)?;
        let bytes_receiver = self.bytes_receiver;
        let transaction_manager_handle = tokio::spawn(async move {
            tracing::info!(target: "archon::transactions", "Spawning transaction manager in new thread...");
            TransactionManager::execute(
                bytes_receiver,
                l1_chain_id,
                l1_batch_inbox_address,
                sender_address,
                private_key,
                provider,
                receiver,
                sender,
            )
            .await
        });
        Ok(transaction_manager_handle)
    }

    /// Sends the given [Transaction] to L1.
    ///
    /// This is used to publish a transaction with incrementally higher gas prices
    /// until the transaction eventually confirms. This method blocks until an
    /// invocation of sendTx returns (called with differing gas prices). The method
    /// may be canceled using the passed context.
    ///
    /// The initially supplied transaction must be signed, have gas estimation done, and have a reasonable gas fee.
    /// When the transaction is resubmitted the tx manager will re-sign the transaction at a different gas pricing
    /// but retain the gas used, the nonce, and the data.
    ///
    /// NOTE: This should be called by AT MOST one caller at a time.
    pub async fn send_transaction(
        provider: Provider<Http>,
        wallet: LocalWallet,
        tx: TransactionRequest,
    ) -> Result<TransactionReceipt> {
        // Lock the send transaction method
        // let lock_result = TRANSACTION_MANAGER_LOCK
        //     .lock()
        //     .map_err(|_| TransactionManagerError::SendTransactionLocked)?;

        // Set the interval on the provider
        // let provider = provider.interval(Duration::from_millis(2000u64));

        // Insert the gas escalator middleware into the provider
        // let provider = {
        //     let escalator = GeometricGasPrice::new(5.0, 10u64, None::<u64>);
        //     GasEscalatorMiddleware::new(provider, escalator, Frequency::PerBlock)
        // };

        // Construct the signer middleware
        let client = SignerMiddleware::new(provider, wallet);

        // Send the transaction
        let pending_tx = client.send_transaction(tx, None).await?;
        let receipt = pending_tx.confirmations(6).await?;
        let receipt =
            receipt.ok_or(TransactionManagerError::TransactionReceiptNotFound)?;

        // Force drop the lock result to demonstrate we are done sending the transaction
        // std::mem::drop(lock_result);

        // Return the receipt
        Ok(receipt)
    }

    /// Crafts a transaction from the given [Bytes].
    /// This queries L1 for the current fee market conditions
    /// as well as for the nonce.
    /// NOTE: This method SHOULD NOT publish the resulting transaction.
    pub async fn craft_transaction(
        l1_chain_id: u64,
        l1_batch_inbox_address: Address,
        sender: Address,
        provider: &Provider<Http>,
        bytes: Bytes,
    ) -> Result<TransactionRequest> {
        // Get the current nonce and gas price
        let nonce = provider.get_transaction_count(sender, None).await?;
        let gas_price = provider.get_gas_price().await?;

        // Create the transaction
        let tx = TransactionRequest::new()
            .chain_id(l1_chain_id)
            .to(l1_batch_inbox_address)
            .data(bytes)
            .gas_price(gas_price)
            .nonce(nonce);

        Ok(tx)
    }
}

impl Stage for TransactionManager {
    type Input = Bytes;
    type Output = TransactionReceipt;
    fn build(
        &mut self,
        pipeline: &mut Archon,
        receiver: Option<Receiver<Pin<Box<Bytes>>>>,
    ) -> Result<Receiver<Pin<Box<TransactionReceipt>>>> {
        let (archon_sender, tx_mgr_receiver) = channel::<Pin<Box<Bytes>>>();
        let (tx_mgr_sender, archon_receiver) = channel::<Pin<Box<TransactionReceipt>>>();
        pipeline.with_tx_manager_sender(archon_sender.clone());
        // self.tx_manager_receiver = Some(archon_receiver.clone());
        // let transaction_manager = pipeline.tx_manager.take();
        let mut transaction_manager = TransactionManager::new(
            Some(pipeline.config().network.into()),
            Some(pipeline.config().batcher_inbox),
            Some(pipeline.config().proposer_address),
            Some(pipeline.config().batcher_private_key.clone()),
            pipeline.config().get_l1_client()?,
        );
        transaction_manager.with_sender(tx_mgr_sender);
        transaction_manager.with_receiver(tx_mgr_receiver);
        transaction_manager.receive_bytes(receiver);

        Ok(archon_receiver)
    }
}
