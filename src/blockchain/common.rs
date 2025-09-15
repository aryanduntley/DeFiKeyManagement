use anyhow::Result;
use crate::blockchain::SupportedBlockchain;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct WalletKeys {
    pub private_key: String,
    pub public_key: String,
    pub address: String,
    pub address_with_checksum: Option<String>,   // New: checksummed version
    pub derivation_path: String,
    pub additional_data: HashMap<String, String>,     // New: blockchain-specific data
    pub secondary_addresses: HashMap<String, String>, // New: secondary addresses
}

impl WalletKeys {
    // Helper constructor for simple wallets without additional data
    pub fn new_simple(
        private_key: String,
        public_key: String,
        address: String,
        derivation_path: String,
    ) -> Self {
        Self {
            private_key,
            public_key,
            address,
            address_with_checksum: None,
            derivation_path,
            additional_data: HashMap::new(),
            secondary_addresses: HashMap::new(),
        }
    }

    // Helper constructor for wallets with checksummed addresses
    pub fn new_with_checksum(
        private_key: String,
        public_key: String,
        address: String,
        address_with_checksum: Option<String>,
        derivation_path: String,
    ) -> Self {
        Self {
            private_key,
            public_key,
            address,
            address_with_checksum,
            derivation_path,
            additional_data: HashMap::new(),
            secondary_addresses: HashMap::new(),
        }
    }

    // Helper methods to add data
    pub fn add_data(&mut self, key: String, value: String) {
        self.additional_data.insert(key, value);
    }

    pub fn add_secondary_address(&mut self, address_type: String, address: String) {
        self.secondary_addresses.insert(address_type, address);
    }
}

pub trait BlockchainHandler {
    fn derive_from_mnemonic(
        &self,
        mnemonic: &str,
        passphrase: Option<&str>,
        account: u32,
        address_index: u32,
        custom_path: Option<&str>,
    ) -> Result<WalletKeys>;
    
    fn derive_from_private_key(&self, private_key: &str) -> Result<WalletKeys>;
    
    fn validate_address(&self, address: &str) -> bool;
    
    fn get_blockchain_name(&self) -> &'static str;
}

pub fn get_blockchain_handler(blockchain: &SupportedBlockchain) -> Result<Box<dyn BlockchainHandler>> {
    Ok(match blockchain {
        SupportedBlockchain::Bitcoin => {
            Box::new(crate::blockchain::bitcoin::BitcoinHandler::new())
        },
        SupportedBlockchain::Ethereum => {
            Box::new(crate::blockchain::ethereum::EthereumHandler::new(blockchain.clone()))
        },
        SupportedBlockchain::Solana => {
            Box::new(crate::blockchain::solana::SolanaHandler::new())
        },
        SupportedBlockchain::Stellar => {
            Box::new(crate::blockchain::stellar::StellarHandler::new())
        },
        // Phase 1 blockchain handlers
        SupportedBlockchain::XRP => {
            Box::new(crate::blockchain::xrp::XrpHandler::new())
        },
        SupportedBlockchain::Litecoin => {
            Box::new(crate::blockchain::litecoin::LitecoinHandler::new())
        },
        SupportedBlockchain::Cardano => {
            Box::new(crate::blockchain::cardano::CardanoHandler::new())
        },
        // Phase 2 blockchain handlers
        SupportedBlockchain::Tron => {
            Box::new(crate::blockchain::tron::TronHandler::new())
        },
        SupportedBlockchain::Polygon => {
            Box::new(crate::blockchain::polygon::PolygonHandler::new())
        },
        SupportedBlockchain::Optimism => {
            Box::new(crate::blockchain::optimism::OptimismHandler::new())
        },
        // Phase 3 blockchain handlers
        SupportedBlockchain::Cronos => {
            Box::new(crate::blockchain::cronos::CronosHandler::new())
        },
        SupportedBlockchain::BinanceBNB => {
            Box::new(crate::blockchain::binance::BinanceHandler::new())
        },
        SupportedBlockchain::Cosmos => {
            Box::new(crate::blockchain::cosmos::CosmosHandler::new())
        },
        // Phase 4 blockchain handlers
        SupportedBlockchain::Algorand => {
            Box::new(crate::blockchain::algorand::AlgorandHandler::new())
        },
        SupportedBlockchain::Hedera => {
            Box::new(crate::blockchain::hedera::HederaHandler::new())
        },
        SupportedBlockchain::Polkadot => {
            Box::new(crate::blockchain::polkadot::PolkadotHandler::new())
        },
        // Phase 5 blockchain handlers
        SupportedBlockchain::Sui => {
            Box::new(crate::blockchain::sui::SuiHandler::new())
        },
        SupportedBlockchain::IOTA => {
            Box::new(crate::blockchain::iota::IotaHandler::new())
        },
        SupportedBlockchain::TON => {
            Box::new(crate::blockchain::ton::TonHandler::new())
        },
        // Phase 6 blockchain handlers
        SupportedBlockchain::XDC => {
            Box::new(crate::blockchain::xdc::XdcHandler::new())
        },
    })
}