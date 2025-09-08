use anyhow::{Result, Context};
use crate::blockchain::{BlockchainHandler, WalletKeys, SupportedBlockchain};
use crate::crypto::bip32::{derive_secp256k1_key_from_mnemonic, private_key_to_public_key_secp256k1};
use bitcoin::secp256k1::{Secp256k1, SecretKey};
use bitcoin::address::Address;
use bitcoin::key::PublicKey;
use bitcoin::Network;
use std::str::FromStr;

pub struct BitcoinHandler {
    network: Network,
}

impl BitcoinHandler {
    pub fn new() -> Self {
        Self {
            network: Network::Bitcoin, // Mainnet
        }
    }
    
    pub fn new_testnet() -> Self {
        Self {
            network: Network::Testnet,
        }
    }
}

impl BlockchainHandler for BitcoinHandler {
    fn derive_from_mnemonic(
        &self,
        mnemonic: &str,
        passphrase: Option<&str>,
        account: u32,
        address_index: u32,
        custom_path: Option<&str>,
    ) -> Result<WalletKeys> {
        let derivation_path = match custom_path {
            Some(path) => path.to_string(),
            None => SupportedBlockchain::Bitcoin.get_default_derivation_path(account, address_index),
        };
        
        // Derive private and public key using BIP-32
        let (private_key_bytes, public_key_bytes) = derive_secp256k1_key_from_mnemonic(
            mnemonic,
            passphrase,
            &derivation_path,
        )?;
        
        // Generate Bitcoin address from public key
        let address = self.public_key_to_address(&public_key_bytes)?;
        
        Ok(WalletKeys {
            private_key: hex::encode(&private_key_bytes),
            public_key: hex::encode(&public_key_bytes),
            address,
            derivation_path,
        })
    }
    
    fn derive_from_private_key(&self, private_key: &str) -> Result<WalletKeys> {
        // Parse private key (remove 0x prefix if present)
        let private_key_hex = if private_key.starts_with("0x") {
            &private_key[2..]
        } else {
            private_key
        };
        
        let private_key_bytes = hex::decode(private_key_hex)
            .context("Invalid hexadecimal private key")?;
        
        if private_key_bytes.len() != 32 {
            return Err(anyhow::anyhow!("Bitcoin private key must be 32 bytes"));
        }
        
        // Derive public key
        let public_key_bytes = private_key_to_public_key_secp256k1(&private_key_bytes)?;
        
        // Generate Bitcoin address
        let address = self.public_key_to_address(&public_key_bytes)?;
        
        Ok(WalletKeys {
            private_key: hex::encode(&private_key_bytes),
            public_key: hex::encode(&public_key_bytes),
            address,
            derivation_path: "imported".to_string(),
        })
    }
    
    fn validate_address(&self, address: &str) -> bool {
        Address::from_str(address).is_ok()
    }
    
    fn get_blockchain_name(&self) -> &'static str {
        "bitcoin"
    }
}

impl BitcoinHandler {
    fn public_key_to_address(&self, public_key_bytes: &[u8]) -> Result<String> {
        // Parse the compressed secp256k1 public key
        let secp_pubkey = bitcoin::secp256k1::PublicKey::from_slice(public_key_bytes)
            .context("Invalid secp256k1 public key format")?;
        
        // Convert to bitcoin::PublicKey
        let public_key = PublicKey::new(secp_pubkey);
        
        // Create P2WPKH (Native SegWit) address - most common modern format
        let address = Address::p2wpkh(&public_key, self.network)
            .context("Failed to create P2WPKH address")?;
        
        Ok(address.to_string())
    }
    
    /// Generate legacy P2PKH address (starts with 1)
    pub fn public_key_to_legacy_address(&self, public_key_bytes: &[u8]) -> Result<String> {
        let secp_pubkey = bitcoin::secp256k1::PublicKey::from_slice(public_key_bytes)
            .context("Invalid secp256k1 public key format")?;
        let public_key = PublicKey::new(secp_pubkey);
        
        let address = Address::p2pkh(&public_key, self.network);
        Ok(address.to_string())
    }
    
    /// Generate P2SH-wrapped SegWit address (starts with 3)
    pub fn public_key_to_nested_segwit_address(&self, public_key_bytes: &[u8]) -> Result<String> {
        let secp_pubkey = bitcoin::secp256k1::PublicKey::from_slice(public_key_bytes)
            .context("Invalid secp256k1 public key format")?;
        let public_key = PublicKey::new(secp_pubkey);
        
        let address = Address::p2shwpkh(&public_key, self.network)
            .context("Failed to create P2SH-WPKH address")?;
        
        Ok(address.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitcoin_from_mnemonic() {
        let handler = BitcoinHandler::new();
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        
        let result = handler.derive_from_mnemonic(mnemonic, None, 0, 0, None);
        assert!(result.is_ok());
        
        let keys = result.unwrap();
        assert!(!keys.private_key.is_empty());
        assert!(!keys.public_key.is_empty());
        assert!(!keys.address.is_empty());
        assert!(keys.address.starts_with("bc1")); // Native SegWit address
    }
    
    #[test]
    fn test_bitcoin_from_private_key() {
        let handler = BitcoinHandler::new();
        let private_key = "0000000000000000000000000000000000000000000000000000000000000001";
        
        let result = handler.derive_from_private_key(private_key);
        assert!(result.is_ok());
        
        let keys = result.unwrap();
        assert_eq!(keys.private_key, private_key);
        assert!(!keys.public_key.is_empty());
        assert!(!keys.address.is_empty());
    }
    
    #[test]
    fn test_address_validation() {
        let handler = BitcoinHandler::new();
        
        // Valid addresses
        assert!(handler.validate_address("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"));
        assert!(handler.validate_address("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"));
        
        // Invalid address
        assert!(!handler.validate_address("invalid_address"));
    }
    
    #[test]
    fn test_address_types() {
        let handler = BitcoinHandler::new();
        let public_key_hex = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
        let public_key_bytes = hex::decode(public_key_hex).unwrap();
        
        // Test different address formats
        let native_segwit = handler.public_key_to_address(&public_key_bytes).unwrap();
        let legacy = handler.public_key_to_legacy_address(&public_key_bytes).unwrap();
        let nested_segwit = handler.public_key_to_nested_segwit_address(&public_key_bytes).unwrap();
        
        assert!(native_segwit.starts_with("bc1"));
        assert!(legacy.starts_with("1"));
        assert!(nested_segwit.starts_with("3"));
    }
}