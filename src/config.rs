use std::{str::FromStr, time::Duration};

use clap::Parser;
use ethers_core::types::{Address, Chain, H256};
use ethers_providers::{Http, Provider};
use eyre::Result;
use serde::{Deserialize, Serialize};

use crate::{errors::ConfigError, extract_env};

/// A system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The private key to use for sequencing.
    pub sequencer_private_key: String,
    /// The public address for sequencing.
    pub sequencer_address: Address,
    /// The private key to use for proposing.
    pub proposer_private_key: String,
    /// The public address for proposing.
    pub proposer_address: Address,
    /// The batcher private key
    pub batcher_private_key: String,
    /// The batcher address
    pub batcher_address: Address,
    /// L1 client rpc url
    pub l1_client_rpc_url: String,
    /// L2 client rpc url
    pub l2_client_rpc_url: String,
    /// The data availability layer to use for batching transactions.
    pub data_availability_layer: u32,
    /// The network to batch transactions for.
    pub network: u32,
    /// The batcher inbox
    pub batcher_inbox: Address,
    /// The driver's polling interval.
    pub polling_interval: Option<Duration>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sequencer_private_key: String::from(
                "0xa0bba68a40ddd0b573c344de2e7dd597af69b3d90e30a87ec91fa0547ddb6ab8",
            ),
            sequencer_address: Address::from_str("0xf4031e0983177452c9e7F27f46ff6bB9CA5933E1")
                .unwrap(),
            proposer_private_key: String::from(
                "0x4a6e5ceb37cd67ed8e740cc25b0ee6d11f6cfabe366daad1c908dec1d178bc72",
            ),
            proposer_address: Address::from_str("0x87A159604e2f18B01a080F672ee011F39777E640")
                .unwrap(),
            batcher_address: Address::from_str("0x7431310e026B69BFC676C0013E12A1A11411EEc9")
                .unwrap(),
            batcher_private_key: String::from(
                "0x4a6e5ceb37cd67ed8e740cc25b0ee6d11f6cfabe366daad1c908dec1d178bc72",
            ),
            l1_client_rpc_url: String::from("http://localhost:8545"),
            l2_client_rpc_url: String::from("http://localhost:8547"),
            data_availability_layer: Chain::from_str("mainnet").unwrap().into(),
            network: Chain::from_str("optimism").unwrap().into(),
            batcher_inbox: Address::from_str("0xff00000000000000000000000000000000042069").unwrap(),
            polling_interval: Some(Duration::from_secs(5)),
        }
    }
}

impl Config {
    /// Parses the CLI sequencer private key string into a 32-byte hash
    pub fn get_sequencer_priv_key(&self) -> H256 {
        H256::from_str(&self.sequencer_private_key).unwrap()
    }

    /// Parses the CLI sequencer address string into an address
    pub fn get_sequencer_address(&self) -> Address {
        self.sequencer_address
    }

    /// Parses the CLI proposer private key string into a 32-byte hash
    pub fn get_proposer_priv_key(&self) -> H256 {
        H256::from_str(&self.proposer_private_key).unwrap()
    }

    /// Parses the CLI proposer address string into an address
    pub fn get_proposer_address(&self) -> Address {
        self.proposer_address
    }

    /// Constructs an L1 provider
    pub fn get_l1_client(&self) -> Result<Provider<Http>> {
        Ok(Provider::<Http>::try_from(&self.l1_client_rpc_url)
            .map_err(|_| ConfigError::InvalidL1ClientUrl)?)
    }

    /// Constructs an L2 provider
    pub fn get_l2_client(&self) -> Result<Provider<Http>> {
        Ok(Provider::<Http>::try_from(&self.l2_client_rpc_url)
            .map_err(|_| ConfigError::InvalidL2ClientUrl)?)
    }
}

/// The Archon CLI
#[derive(Parser)]
pub struct Cli {
    /// The private key to use for sequencing.
    /// If not provided, a fully public private key will be used as the default.
    /// The default private key is _only_ recommended for testing purposes.
    #[clap(
        short = 'k',
        long,
        default_value = "0xa0bba68a40ddd0b573c344de2e7dd597af69b3d90e30a87ec91fa0547ddb6ab8"
    )]
    sequencer_private_key: String,
    /// The sequencer public address.
    #[clap(
        short = 's',
        long,
        default_value = "0xf4031e0983177452c9e7F27f46ff6bB9CA5933E1"
    )]
    sequencer_address: String,
    /// The private key to use for proposing.
    #[clap(
        short = 'p',
        long,
        default_value = "0x4a6e5ceb37cd67ed8e740cc25b0ee6d11f6cfabe366daad1c908dec1d178bc72"
    )]
    proposer_private_key: String,
    /// The proposer public address.
    #[clap(
        short = 'a',
        long,
        default_value = "0x87A159604e2f18B01a080F672ee011F39777E640"
    )]
    proposer_address: String,
    /// The private key to use for batching.
    #[clap(
        short = 'x',
        long,
        default_value = "0x4a6e5ceb37cd67ed8e740cc25b0ee6d11f6cfabe366daad1c908dec1d178bc72"
    )]
    batcher_private_key: String,
    /// The batcher public address.
    #[clap(
        short = 'q',
        long,
        default_value = "0x87A159604e2f18B01a080F672ee011F39777E640"
    )]
    batcher_address: String,
    /// Batcher inbox address.
    #[clap(
        short = 'b',
        long,
        default_value = "0xff00000000000000000000000000000000042069"
    )]
    batcher_inbox: String,
    /// The L1 client rpc url
    #[clap(short = 'l', long)]
    l1_client_rpc_url: Option<String>,
    /// The L2 client rpc url
    #[clap(short = 'c', long)]
    l2_client_rpc_url: Option<String>,
    /// The data availability layer to use for batching transactions.
    #[clap(short = 'd', long, default_value = "mainnet")]
    data_availability_layer: String,
    /// The network to batch transactions for.
    #[clap(short = 'n', long, default_value = "optimism")]
    network: String,
    /// The driver's polling interval.
    #[clap(short = 'i', long, default_value = "5")]
    polling_interval: u64,
}

impl Cli {
    /// Convert the CLI arguments into a config
    pub fn to_config(self) -> Config {
        // Parse optional url params
        let l1_rpc_url = self.l1_client_rpc_url.unwrap_or(extract_env!("L1_RPC_URL"));
        let l2_rpc_url = self.l2_client_rpc_url.unwrap_or(extract_env!("L2_RPC_URL"));

        // let config_path = home_dir().unwrap().join(".archon/archon.toml");
        Config {
            sequencer_private_key: self.sequencer_private_key,
            sequencer_address: Address::from_str(&self.sequencer_address).unwrap_or_default(),
            proposer_private_key: self.proposer_private_key,
            proposer_address: Address::from_str(&self.proposer_address).unwrap_or_default(),
            batcher_private_key: self.batcher_private_key,
            batcher_address: Address::from_str(&self.batcher_address).unwrap_or_default(),
            l1_client_rpc_url: l1_rpc_url,
            l2_client_rpc_url: l2_rpc_url,
            data_availability_layer: Chain::from_str(&self.data_availability_layer)
                .unwrap()
                .into(),
            network: Chain::from_str(&self.network).unwrap().into(),
            polling_interval: Some(Duration::from_secs(self.polling_interval)),
            batcher_inbox: Address::from_str(&self.batcher_inbox).unwrap(),
        }
    }
}
