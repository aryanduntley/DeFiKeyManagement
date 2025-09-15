use anyhow::{Result, Context};
use crate::blockchain::{BlockchainHandler, WalletKeys};
use crate::crypto::ed25519_utils::{derive_ed25519_key_from_mnemonic, private_key_to_public_key_ed25519};
use sui_sdk_types::{Address as SuiAddress, Ed25519PublicKey};

pub struct SuiHandler;

impl SuiHandler {
    pub fn new() -> Self {
        Self
    }

    fn public_key_to_address(&self, public_key_bytes: &[u8]) -> Result<String> {
        if public_key_bytes.len() != 32 {
            anyhow::bail!("Sui public key must be 32 bytes, got {}", public_key_bytes.len());
        }

        // Sui address generation using sui-sdk-types:
        // 1. Create Ed25519PublicKey from raw bytes
        // 2. Use derive_address() method to get proper Sui address
        // 3. Convert to string representation
        //
        // According to Sui documentation:
        // Account addresses are cryptographically derived from authenticators using
        // Blake2b256 hash of the sequence: signature_scheme_flag || authenticator_bytes

        // Create Ed25519PublicKey from raw bytes
        let sui_public_key = Ed25519PublicKey::new(public_key_bytes.try_into()
            .map_err(|_| anyhow::anyhow!("Invalid public key length"))?);

        // Use the official derive_address method from Ed25519PublicKey
        let sui_address = sui_public_key.derive_address();

        // Convert to hex string representation
        Ok(sui_address.to_string())
    }

    fn validate_sui_address(&self, address: &str) -> bool {
        // Sui address validation using sui-sdk-types:
        // 1. Try to parse the address string into a SuiAddress
        // 2. If parsing succeeds, the address is valid
        // 3. Return validation result

        match address.parse::<SuiAddress>() {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

impl BlockchainHandler for SuiHandler {
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
                // Sui uses SLIP-0010 derivation path: m/44'/784'/0'/0'/0'
                format!("m/44'/784'/{}'/{}'/{}'", account, 0, address_index)
            },
        };

        // Derive ed25519 private and public keys using SLIP-0010
        let (private_key_bytes, public_key_bytes) = derive_ed25519_key_from_mnemonic(
            mnemonic,
            passphrase,
            &derivation_path,
        )?;

        // Generate Sui address from public key
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
            anyhow::bail!("Sui private key must be 32 bytes");
        }

        // Derive public key from private key
        let public_key_bytes = private_key_to_public_key_ed25519(&private_key_bytes)?;

        // Generate Sui address from public key
        let address = self.public_key_to_address(&public_key_bytes)?;

        Ok(WalletKeys::new_simple(
            hex::encode(&private_key_bytes),
            hex::encode(&public_key_bytes),
            address,
            "Imported from private key".to_string(),
        ))
    }

    fn validate_address(&self, address: &str) -> bool {
        // Use Sui-specific address validation
        self.validate_sui_address(address)
    }

    fn get_blockchain_name(&self) -> &'static str {
        "Sui"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sui_address_validation() {
        let handler = SuiHandler::new();

        // Generate a real Sui address to test validation
        let private_key = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let result = handler.derive_from_private_key(private_key);
        assert!(result.is_ok());
        let wallet_keys = result.unwrap();

        // Test with the generated address
        assert!(handler.validate_address(&wallet_keys.address));

        // Test with known invalid addresses
        assert!(!handler.validate_address("invalid_address"));
        assert!(!handler.validate_address("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2")); // Bitcoin address
        assert!(!handler.validate_address("rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH")); // XRP address
        assert!(!handler.validate_address("")); // Empty string
        assert!(!handler.validate_address("sui")); // Too short
    }

    #[test]
    fn test_sui_private_key_derivation() {
        let handler = SuiHandler::new();
        let private_key = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

        let result = handler.derive_from_private_key(private_key);
        assert!(result.is_ok());

        let keys = result.unwrap();
        assert_eq!(keys.private_key, private_key);
        assert!(keys.public_key.len() == 64); // 32 bytes in hex
        assert!(handler.validate_address(&keys.address));

        // Print the generated address for debugging
        println!("Generated Sui address: {}", keys.address);

        // Verify Sui address format (0x prefix and appropriate length)
        assert!(keys.address.starts_with("0x"));
        assert!(keys.address.len() >= 40);
    }
}