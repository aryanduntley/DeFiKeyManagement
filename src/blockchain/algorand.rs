use anyhow::{Result, Context};
use crate::blockchain::{BlockchainHandler, WalletKeys};
use crate::crypto::ed25519_utils::{derive_ed25519_key_from_mnemonic, private_key_to_public_key_ed25519};
use algo_rust_sdk::crypto::Address as AlgorandAddress;

pub struct AlgorandHandler;

impl AlgorandHandler {
    pub fn new() -> Self {
        Self
    }

    fn public_key_to_address(&self, public_key_bytes: &[u8]) -> Result<String> {
        // Algorand address generation using algo_rust_sdk:
        // 1. Take the full 32-byte ed25519 public key
        // 2. Create an Algorand Address from the public key bytes
        // 3. Encode to base32 string with checksum

        if public_key_bytes.len() != 32 {
            anyhow::bail!("Algorand public key must be 32 bytes, got {}", public_key_bytes.len());
        }

        // Convert public key bytes to [u8; 32] array
        let mut public_key_array = [0u8; 32];
        public_key_array.copy_from_slice(public_key_bytes);

        // Create Algorand Address from public key bytes
        let algorand_address = AlgorandAddress::new(public_key_array);

        // Encode address to base32 string with checksum
        let address_string = algorand_address.encode_string();

        Ok(address_string)
    }
}

impl BlockchainHandler for AlgorandHandler {
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
            None => {
                // Algorand uses SLIP-0010 derivation path: m/44'/283'/0'/0'/0'
                format!("m/44'/283'/{}'/{}'/{}'", account, 0, address_index)
            },
        };

        // Derive ed25519 private and public keys using SLIP-0010
        let (private_key_bytes, public_key_bytes) = derive_ed25519_key_from_mnemonic(
            mnemonic,
            passphrase,
            &derivation_path,
        )?;

        // Generate Algorand address from public key
        let address = self.public_key_to_address(&public_key_bytes)?;

        Ok(WalletKeys::new_simple(
            hex::encode(&private_key_bytes),
            hex::encode(&public_key_bytes),
            address,
            derivation_path,
        ))
    }

    fn derive_from_private_key(&self, private_key_hex: &str) -> Result<WalletKeys> {
        // Parse private key from hex
        let private_key_bytes = hex::decode(private_key_hex)
            .context("Invalid hex private key")?;

        if private_key_bytes.len() != 32 {
            anyhow::bail!("Algorand private key must be 32 bytes");
        }

        // Derive public key from private key
        let public_key_bytes = private_key_to_public_key_ed25519(&private_key_bytes)?;

        // Generate Algorand address from public key
        let address = self.public_key_to_address(&public_key_bytes)?;

        Ok(WalletKeys::new_simple(
            hex::encode(&private_key_bytes),
            hex::encode(&public_key_bytes),
            address,
            "Imported from private key".to_string(),
        ))
    }

    fn validate_address(&self, address: &str) -> bool {
        // Use algo_rust_sdk Address::from_string for proper validation
        // This validates the base32 encoding and checksum
        AlgorandAddress::from_string(address).is_ok()
    }

    fn get_blockchain_name(&self) -> &'static str {
        "Algorand"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_algorand_address_validation() {
        let handler = AlgorandHandler::new();

        // Generate a real Algorand address to test validation
        let private_key = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let result = handler.derive_from_private_key(private_key);
        assert!(result.is_ok());
        let wallet_keys = result.unwrap();

        // Test with the generated address
        assert!(handler.validate_address(&wallet_keys.address));

        // Test with known valid Algorand addresses (these should be real valid addresses)
        // Note: These are example Algorand addresses from the documentation

        // Test invalid addresses
        assert!(!handler.validate_address("invalid_address"));
        assert!(!handler.validate_address("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2")); // Bitcoin address
        assert!(!handler.validate_address("0x742d35Cc6634C0532925a3b8D432C3475CB978B3")); // Ethereum address
        assert!(!handler.validate_address("")); // Empty string
        assert!(!handler.validate_address("algo")); // Too short
    }

    #[test]
    fn test_algorand_private_key_derivation() {
        let handler = AlgorandHandler::new();
        let private_key = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

        let result = handler.derive_from_private_key(private_key);
        assert!(result.is_ok());

        let keys = result.unwrap();
        assert_eq!(keys.private_key, private_key);
        assert!(keys.public_key.len() == 64); // 32 bytes in hex
        assert!(handler.validate_address(&keys.address));

        // Print the generated address for debugging
        println!("Generated Algorand address: {}", keys.address);

        // Verify address format (Algorand addresses are 58 characters, base32)
        assert_eq!(keys.address.len(), 58);
        assert!(keys.address.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()));
    }
}