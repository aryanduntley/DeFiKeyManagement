use anyhow::{Result, Context};
use crate::blockchain::{BlockchainHandler, WalletKeys, SupportedBlockchain};
use crate::crypto::ed25519_utils::{derive_ed25519_key_from_mnemonic, private_key_to_public_key_ed25519};
use ed25519_dalek::{SigningKey, VerifyingKey};
use bs58;

pub struct SolanaHandler;

impl SolanaHandler {
    pub fn new() -> Self {
        Self
    }
}

impl BlockchainHandler for SolanaHandler {
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
            None => SupportedBlockchain::Solana.get_default_derivation_path(account, address_index),
        };
        
        // Derive ed25519 keys using SLIP-0010
        let (private_key_bytes, public_key_bytes) = derive_ed25519_key_from_mnemonic(
            mnemonic,
            passphrase,
            &derivation_path,
        )?;
        
        // For Solana, the address is the base58-encoded public key
        let address = bs58::encode(&public_key_bytes).into_string();
        
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
            return Err(anyhow::anyhow!("Solana private key must be 32 bytes"));
        }
        
        // Derive public key
        let public_key_bytes = private_key_to_public_key_ed25519(&private_key_bytes)?;
        
        // Generate Solana address (base58-encoded public key)
        let address = bs58::encode(&public_key_bytes).into_string();
        
        Ok(WalletKeys {
            private_key: hex::encode(&private_key_bytes),
            public_key: hex::encode(&public_key_bytes),
            address,
            derivation_path: "imported".to_string(),
        })
    }
    
    fn validate_address(&self, address: &str) -> bool {
        // Solana addresses are 32-byte public keys encoded in base58
        match bs58::decode(address).into_vec() {
            Ok(decoded) => decoded.len() == 32,
            Err(_) => false,
        }
    }
    
    fn get_blockchain_name(&self) -> &'static str {
        "solana"
    }
}

impl SolanaHandler {
    /// Convert private key bytes to Solana's keypair format (private + public key concatenated)
    pub fn private_key_to_keypair_bytes(&self, private_key_bytes: &[u8]) -> Result<Vec<u8>> {
        if private_key_bytes.len() != 32 {
            return Err(anyhow::anyhow!("Invalid private key length for Solana keypair"));
        }
        
        let public_key_bytes = private_key_to_public_key_ed25519(private_key_bytes)?;
        
        // Solana keypair format: 64 bytes (32 private + 32 public)
        let mut keypair = Vec::with_capacity(64);
        keypair.extend_from_slice(private_key_bytes);
        keypair.extend_from_slice(&public_key_bytes);
        
        Ok(keypair)
    }
    
    /// Generate Solana keypair in the format expected by Solana CLI tools
    pub fn to_solana_cli_format(&self, private_key_bytes: &[u8]) -> Result<String> {
        let keypair_bytes = self.private_key_to_keypair_bytes(private_key_bytes)?;
        
        // Solana CLI expects JSON array format
        let json_array: Vec<u8> = keypair_bytes;
        Ok(serde_json::to_string(&json_array)?)
    }
    
    /// Parse Solana CLI keypair format back to private key
    pub fn from_solana_cli_format(&self, keypair_json: &str) -> Result<Vec<u8>> {
        let keypair_array: Vec<u8> = serde_json::from_str(keypair_json)
            .context("Invalid Solana CLI keypair format")?;
        
        if keypair_array.len() != 64 {
            return Err(anyhow::anyhow!("Invalid Solana keypair length"));
        }
        
        // Return just the private key portion (first 32 bytes)
        Ok(keypair_array[0..32].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solana_from_mnemonic() {
        let handler = SolanaHandler::new();
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        
        let result = handler.derive_from_mnemonic(mnemonic, None, 0, 0, None);
        assert!(result.is_ok());
        
        let keys = result.unwrap();
        assert!(!keys.private_key.is_empty());
        assert!(!keys.public_key.is_empty());
        assert!(!keys.address.is_empty());
        
        // Solana addresses should be valid base58
        assert!(handler.validate_address(&keys.address));
    }
    
    #[test]
    fn test_solana_from_private_key() {
        let handler = SolanaHandler::new();
        let private_key = "0000000000000000000000000000000000000000000000000000000000000001";
        
        let result = handler.derive_from_private_key(private_key);
        assert!(result.is_ok());
        
        let keys = result.unwrap();
        assert_eq!(keys.private_key, private_key);
        assert!(!keys.public_key.is_empty());
        assert!(!keys.address.is_empty());
        assert!(handler.validate_address(&keys.address));
    }
    
    #[test]
    fn test_address_validation() {
        let handler = SolanaHandler::new();
        
        // Valid Solana address (example)
        let valid_address = "11111111111111111111111111111111";
        assert!(handler.validate_address(valid_address));
        
        // Invalid addresses
        assert!(!handler.validate_address("invalid_address"));
        assert!(!handler.validate_address("too_short"));
        assert!(!handler.validate_address("0x742d35Cc6634C0532925a3b8D322C8e1c6a331cb")); // ETH address
    }
    
    #[test]
    fn test_keypair_conversion() {
        let handler = SolanaHandler::new();
        let private_key = hex::decode("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        
        // Test keypair bytes generation
        let keypair_bytes = handler.private_key_to_keypair_bytes(&private_key);
        assert!(keypair_bytes.is_ok());
        
        let keypair = keypair_bytes.unwrap();
        assert_eq!(keypair.len(), 64);
        assert_eq!(&keypair[0..32], &private_key);
        
        // Test CLI format conversion
        let cli_format = handler.to_solana_cli_format(&private_key);
        assert!(cli_format.is_ok());
        
        let cli_json = cli_format.unwrap();
        let parsed_private = handler.from_solana_cli_format(&cli_json);
        assert!(parsed_private.is_ok());
        assert_eq!(parsed_private.unwrap(), private_key);
    }
    
    #[test]
    fn test_derivation_path() {
        let handler = SolanaHandler::new();
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        
        // Test custom path
        let custom_path = "m/44'/501'/1'/0'";
        let result = handler.derive_from_mnemonic(mnemonic, None, 0, 0, Some(custom_path));
        assert!(result.is_ok());
        
        let keys = result.unwrap();
        assert_eq!(keys.derivation_path, custom_path);
    }
}