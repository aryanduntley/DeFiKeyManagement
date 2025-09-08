use anyhow::{Result, Context};
use crate::blockchain::{BlockchainHandler, WalletKeys, SupportedBlockchain};
use crate::crypto::ed25519_utils::{derive_ed25519_key_from_mnemonic, private_key_to_public_key_ed25519};
use stellar_base::crypto::PublicKey as StellarPublicKey;

pub struct StellarHandler;

impl StellarHandler {
    pub fn new() -> Self {
        Self
    }
}

impl BlockchainHandler for StellarHandler {
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
            None => SupportedBlockchain::Stellar.get_default_derivation_path(account, address_index),
        };
        
        // Derive ed25519 keys using SLIP-0010
        let (private_key_bytes, public_key_bytes) = derive_ed25519_key_from_mnemonic(
            mnemonic,
            passphrase,
            &derivation_path,
        )?;
        
        // Generate Stellar address from public key
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
            return Err(anyhow::anyhow!("Stellar private key must be 32 bytes"));
        }
        
        // Derive public key
        let public_key_bytes = private_key_to_public_key_ed25519(&private_key_bytes)?;
        
        // Generate Stellar address
        let address = self.public_key_to_address(&public_key_bytes)?;
        
        Ok(WalletKeys {
            private_key: hex::encode(&private_key_bytes),
            public_key: hex::encode(&public_key_bytes),
            address,
            derivation_path: "imported".to_string(),
        })
    }
    
    fn validate_address(&self, address: &str) -> bool {
        // Use stellar-base library to validate address
        StellarPublicKey::from_account_id(address).is_ok()
    }
    
    fn get_blockchain_name(&self) -> &'static str {
        "stellar"
    }
}

impl StellarHandler {
    fn public_key_to_address(&self, public_key_bytes: &[u8]) -> Result<String> {
        if public_key_bytes.len() != 32 {
            return Err(anyhow::anyhow!("Invalid public key length for Stellar"));
        }
        
        // Convert bytes to stellar PublicKey
        let stellar_public_key = StellarPublicKey::from_slice(public_key_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to create Stellar public key: {}", e))?;
        
        // Get account ID from public key
        Ok(stellar_public_key.account_id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stellar_from_mnemonic() {
        let handler = StellarHandler::new();
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        
        let result = handler.derive_from_mnemonic(mnemonic, None, 0, 0, None);
        assert!(result.is_ok());
        
        let keys = result.unwrap();
        assert!(!keys.private_key.is_empty());
        assert!(!keys.public_key.is_empty());
        assert!(keys.address.starts_with('G'));
        assert_eq!(keys.address.len(), 56);
        assert!(handler.validate_address(&keys.address));
    }
    
    #[test]
    fn test_stellar_from_private_key() {
        let handler = StellarHandler::new();
        let private_key = "0000000000000000000000000000000000000000000000000000000000000001";
        
        let result = handler.derive_from_private_key(private_key);
        assert!(result.is_ok());
        
        let keys = result.unwrap();
        assert_eq!(keys.private_key, private_key);
        assert!(!keys.public_key.is_empty());
        assert!(keys.address.starts_with('G'));
        assert!(handler.validate_address(&keys.address));
    }
    
    #[test]
    fn test_address_validation() {
        let handler = StellarHandler::new();
        
        // Invalid addresses
        assert!(!handler.validate_address("invalid_address"));
        assert!(!handler.validate_address("0x742d35Cc6634C0532925a3b8D322C8e1c6a331cb")); // ETH address
        assert!(!handler.validate_address("GAAAAA")); // Too short
        
        // Test with a generated address
        let private_key = "1111111111111111111111111111111111111111111111111111111111111111";
        let result = handler.derive_from_private_key(private_key);
        assert!(result.is_ok());
        
        let keys = result.unwrap();
        assert!(handler.validate_address(&keys.address));
    }
    
    #[test]
    fn test_stellar_base_integration() {
        let handler = StellarHandler::new();
        let private_key = "2222222222222222222222222222222222222222222222222222222222222222";
        
        let keys = handler.derive_from_private_key(private_key).unwrap();
        
        // Test that the address is valid using stellar-base validation
        assert!(handler.validate_address(&keys.address));
        
        // Test that the address has the correct Stellar format
        assert!(keys.address.starts_with('G'));
        assert_eq!(keys.address.len(), 56);
    }
}