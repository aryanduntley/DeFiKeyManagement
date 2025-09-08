use anyhow::Result;
use crate::blockchain::SupportedBlockchain;

#[derive(Debug, Clone)]
pub struct WalletKeys {
    pub private_key: String,
    pub public_key: String,
    pub address: String,
    pub derivation_path: String,
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
        SupportedBlockchain::Ethereum | 
        SupportedBlockchain::Polygon |
        SupportedBlockchain::Cronos |
        SupportedBlockchain::Optimism |
        SupportedBlockchain::Quant => {
            Box::new(crate::blockchain::ethereum::EthereumHandler::new(blockchain.clone()))
        },
        SupportedBlockchain::Solana => {
            Box::new(crate::blockchain::solana::SolanaHandler::new())
        },
        SupportedBlockchain::Stellar => {
            Box::new(crate::blockchain::stellar::StellarHandler::new())
        },
        // Cardano and other chains - placeholder for now
        _ => {
            // For now, unsupported chains will use a placeholder
            // TODO: Implement remaining blockchain handlers
            return Err(anyhow::anyhow!("Blockchain handler not yet implemented: {:?}", blockchain));
        }
    })
}