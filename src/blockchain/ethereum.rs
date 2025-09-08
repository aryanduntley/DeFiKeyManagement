use anyhow::{Result, Context};
use crate::blockchain::{BlockchainHandler, WalletKeys, SupportedBlockchain};
use crate::crypto::bip32::{derive_secp256k1_key_from_mnemonic, private_key_to_public_key_secp256k1};
use k256::ecdsa::SigningKey;
use sha3::{Keccak256, Digest};
use std::str::FromStr;

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
        if !address.starts_with("0x") || address.len() != 42 {
            return false;
        }
        
        // Check if it's valid hex
        hex::decode(&address[2..]).is_ok()
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
        if public_key_bytes.len() != 33 && public_key_bytes.len() != 65 {
            return Err(anyhow::anyhow!("Invalid public key length for Ethereum"));
        }
        
        // Convert compressed public key to uncompressed if needed
        let uncompressed_key = if public_key_bytes.len() == 33 {
            self.decompress_public_key(public_key_bytes)?
        } else {
            public_key_bytes.to_vec()
        };
        
        // Skip the first byte (0x04) and take the remaining 64 bytes
        if uncompressed_key.len() != 65 || uncompressed_key[0] != 0x04 {
            return Err(anyhow::anyhow!("Invalid uncompressed public key format"));
        }
        
        let key_bytes = &uncompressed_key[1..];
        
        // Compute Keccak256 hash of the public key
        let mut hasher = Keccak256::new();
        hasher.update(key_bytes);
        let hash = hasher.finalize();
        
        // Take the last 20 bytes of the hash as the address
        let address_bytes = &hash[12..];
        let address = format!("0x{}", hex::encode(address_bytes));
        
        // Return checksummed address
        Ok(self.to_checksum_address(&address))
    }
    
    fn decompress_public_key(&self, compressed_key: &[u8]) -> Result<Vec<u8>> {
        use k256::elliptic_curve::sec1::FromEncodedPoint;
        use k256::elliptic_curve::sec1::ToEncodedPoint;
        use k256::PublicKey;
        
        let public_key = PublicKey::from_sec1_bytes(compressed_key)
            .context("Failed to parse compressed public key")?;
        
        let uncompressed = public_key.to_encoded_point(false);
        Ok(uncompressed.as_bytes().to_vec())
    }
    
    /// Convert address to EIP-55 checksummed format
    fn to_checksum_address(&self, address: &str) -> String {
        let address_lower = address.to_lowercase();
        let address_hex = if address_lower.starts_with("0x") {
            &address_lower[2..]
        } else {
            &address_lower
        };
        
        let mut hasher = Keccak256::new();
        hasher.update(address_hex.as_bytes());
        let hash = hasher.finalize();
        
        let mut checksummed = String::from("0x");
        for (i, c) in address_hex.chars().enumerate() {
            if c.is_ascii_alphabetic() {
                let hash_byte = hash[i / 2];
                let nibble = if i % 2 == 0 { hash_byte >> 4 } else { hash_byte & 0xf };
                if nibble >= 8 {
                    checksummed.push(c.to_ascii_uppercase());
                } else {
                    checksummed.push(c);
                }
            } else {
                checksummed.push(c);
            }
        }
        
        checksummed
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
        
        // Valid addresses
        assert!(handler.validate_address("0x742d35Cc6634C0532925a3b8D322C8e1c6a331cb"));
        assert!(handler.validate_address("0x0000000000000000000000000000000000000000"));
        
        // Invalid addresses
        assert!(!handler.validate_address("invalid_address"));
        assert!(!handler.validate_address("0x123")); // Too short
        assert!(!handler.validate_address("742d35Cc6634C0532925a3b8D322C8e1c6a331cb")); // Missing 0x
    }
    
    #[test]
    fn test_checksum_address() {
        let handler = EthereumHandler::new(SupportedBlockchain::Ethereum);
        let address = "0x742d35cc6634c0532925a3b8d322c8e1c6a331cb";
        let checksummed = handler.to_checksum_address(address);
        
        // Should have mixed case for checksum
        assert!(checksummed.contains(char::is_uppercase));
        assert!(checksummed.contains(char::is_lowercase));
        assert_eq!(checksummed.len(), 42);
        assert!(checksummed.starts_with("0x"));
    }
}