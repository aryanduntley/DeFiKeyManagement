use anyhow::{Result, Context};
use crate::blockchain::{BlockchainHandler, WalletKeys, SupportedBlockchain};
use crate::crypto::bip32::{derive_secp256k1_key_from_mnemonic, private_key_to_public_key_secp256k1};
use alloy_primitives::{Address, keccak256};
use k256::ecdsa::VerifyingKey;
use k256::elliptic_curve::sec1::ToEncodedPoint;

pub struct EthereumHandler {
    blockchain: SupportedBlockchain,
}

impl EthereumHandler {
    pub fn new(blockchain: SupportedBlockchain) -> Self {
        Self { blockchain }
    }
}

impl BlockchainHandler for EthereumHandler {
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
            None => self.blockchain.get_default_derivation_path(account, address_index),
        };
        
        // Derive private and public key using BIP-32
        let (private_key_bytes, public_key_bytes) = derive_secp256k1_key_from_mnemonic(
            mnemonic,
            passphrase,
            &derivation_path,
        )?;
        
        // Generate Ethereum address from public key
        let address = self.public_key_to_address(&public_key_bytes)?;
        
        Ok(WalletKeys {
            private_key: format!("0x{}", hex::encode(&private_key_bytes)),
            public_key: format!("0x{}", hex::encode(&public_key_bytes)),
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
            return Err(anyhow::anyhow!("Ethereum private key must be 32 bytes"));
        }
        
        // Derive public key
        let public_key_bytes = private_key_to_public_key_secp256k1(&private_key_bytes)?;
        
        // Generate Ethereum address
        let address = self.public_key_to_address(&public_key_bytes)?;
        
        Ok(WalletKeys {
            private_key: format!("0x{}", hex::encode(&private_key_bytes)),
            public_key: format!("0x{}", hex::encode(&public_key_bytes)),
            address,
            derivation_path: "imported".to_string(),
        })
    }
    
    fn validate_address(&self, address: &str) -> bool {
        // Use alloy-primitives Address parsing for proper validation
        Address::parse_checksummed(address, None).is_ok()
    }
    
    fn get_blockchain_name(&self) -> &'static str {
        match self.blockchain {
            SupportedBlockchain::Ethereum => "ethereum",
            SupportedBlockchain::Polygon => "polygon",
            SupportedBlockchain::Cronos => "cronos",
            SupportedBlockchain::Optimism => "optimism",
            SupportedBlockchain::Quant => "quant",
            _ => "ethereum", // Default
        }
    }
}

impl EthereumHandler {
    fn public_key_to_address(&self, public_key_bytes: &[u8]) -> Result<String> {
        // Convert compressed public key to uncompressed format using k256
        let verifying_key = VerifyingKey::from_sec1_bytes(public_key_bytes)
            .context("Invalid public key for Ethereum address generation")?;

        let uncompressed_point = verifying_key.to_encoded_point(false);
        let uncompressed_bytes = uncompressed_point.as_bytes();

        // Remove the 0x04 prefix (first byte) to get the 64-byte coordinate pair
        let public_key_hash_input = &uncompressed_bytes[1..];

        // Compute Keccak256 hash using alloy-primitives
        let hash = keccak256(public_key_hash_input);

        // Take the last 20 bytes as the address
        let address_bytes: [u8; 20] = hash[12..].try_into()
            .context("Failed to extract address bytes from hash")?;

        // Create Address using alloy-primitives (automatically applies EIP-55 checksum)
        let address = Address::from(address_bytes);

        Ok(address.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ethereum_from_mnemonic() {
        let handler = EthereumHandler::new(SupportedBlockchain::Ethereum);
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        
        let result = handler.derive_from_mnemonic(mnemonic, None, 0, 0, None);
        assert!(result.is_ok());
        
        let keys = result.unwrap();
        assert!(keys.private_key.starts_with("0x"));
        assert!(keys.public_key.starts_with("0x"));
        assert!(keys.address.starts_with("0x"));
        assert_eq!(keys.address.len(), 42);
    }
    
    #[test]
    fn test_ethereum_from_private_key() {
        let handler = EthereumHandler::new(SupportedBlockchain::Ethereum);
        let private_key = "0x0000000000000000000000000000000000000000000000000000000000000001";
        
        let result = handler.derive_from_private_key(private_key);
        assert!(result.is_ok());
        
        let keys = result.unwrap();
        assert_eq!(keys.private_key, private_key);
        assert!(keys.public_key.starts_with("0x"));
        assert!(keys.address.starts_with("0x"));
        assert_eq!(keys.address.len(), 42);
    }
    
    #[test]
    fn test_address_validation() {
        let handler = EthereumHandler::new(SupportedBlockchain::Ethereum);
        
        // Valid addresses with proper EIP-55 checksums
        assert!(handler.validate_address("0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed"));
        assert!(handler.validate_address("0x0000000000000000000000000000000000000000")); // All zeros
        
        // Invalid addresses
        assert!(!handler.validate_address("invalid_address"));
        assert!(!handler.validate_address("0x123")); // Too short
        assert!(!handler.validate_address("742d35Cc6634C0532925a3b8D322C8e1c6a331cb")); // Missing 0x
    }
    
    #[test]
    fn test_address_checksum() {
        let handler = EthereumHandler::new(SupportedBlockchain::Ethereum);

        // Test with a known private key to generate an address
        let private_key = "1e99423a4ed27608a15a2616a2b0e9e52ced330ac530edcc32c8ffc6a526aedd";
        let result = handler.derive_from_private_key(private_key);
        assert!(result.is_ok());

        let keys = result.unwrap();
        println!("Generated Ethereum address: {}", keys.address);

        // Address should have proper EIP-55 checksum (mixed case)
        assert!(keys.address.chars().skip(2).any(|c| c.is_ascii_uppercase()));
        assert!(keys.address.chars().skip(2).any(|c| c.is_ascii_lowercase()));
        assert_eq!(keys.address.len(), 42);
        assert!(keys.address.starts_with("0x"));

        // Should validate correctly
        assert!(handler.validate_address(&keys.address));
    }
}