use anyhow::{Result, Context};
use crate::blockchain::{BlockchainHandler, WalletKeys};
use crate::crypto::bip32::{derive_secp256k1_key_from_mnemonic, private_key_to_public_key_secp256k1};
use cosmrs::{AccountId, crypto::{PublicKey as CosmosPublicKey, secp256k1::{SigningKey as CosmosSigningKey, VerifyingKey as CosmosVerifyingKey}}};
use std::str::FromStr;

pub struct CosmosHandler;

impl CosmosHandler {
    pub fn new() -> Self {
        Self
    }

    fn public_key_to_address(&self, public_key_bytes: &[u8]) -> Result<String> {
        // Cosmos uses secp256k1 public keys with Bech32 encoding:
        // 1. Take the compressed public key (33 bytes)
        // 2. Create a cosmos VerifyingKey from the public key bytes
        // 3. Create cosmrs PublicKey and generate AccountId with 'cosmos' prefix

        // Create cosmrs VerifyingKey from compressed secp256k1 public key bytes
        let verifying_key = CosmosVerifyingKey::from_sec1_bytes(public_key_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to create Cosmos verifying key: {}", e))?;

        // Create cosmrs PublicKey from the VerifyingKey
        let public_key = CosmosPublicKey::from(verifying_key);

        // Generate AccountId with "cosmos" prefix (default for Cosmos Hub)
        let account_id = public_key.account_id("cosmos")
            .map_err(|e| anyhow::anyhow!("Failed to create Cosmos account ID: {}", e))?;

        // Convert to string
        Ok(account_id.to_string())
    }
}

impl BlockchainHandler for CosmosHandler {
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
                // Cosmos uses BIP-44 derivation path: m/44'/118'/0'/0/0
                format!("m/44'/118'/{}'/{}/{}", account, 0, address_index)
            },
        };

        // Derive private and public key using BIP-32 (secp256k1 curve)
        let (private_key_bytes, public_key_bytes) = derive_secp256k1_key_from_mnemonic(
            mnemonic,
            passphrase,
            &derivation_path,
        )?;

        // Generate Cosmos address from public key (Bech32 format)
        let address = self.public_key_to_address(&public_key_bytes)?;

        Ok(WalletKeys {
            address,
            public_key: hex::encode(&public_key_bytes),
            private_key: hex::encode(&private_key_bytes),
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
            return Err(anyhow::anyhow!("Cosmos private key must be 32 bytes"));
        }

        // Derive public key
        let public_key_bytes = private_key_to_public_key_secp256k1(&private_key_bytes)?;

        // Generate Cosmos address
        let address = self.public_key_to_address(&public_key_bytes)?;

        Ok(WalletKeys {
            address,
            public_key: hex::encode(&public_key_bytes),
            private_key: hex::encode(&private_key_bytes),
            derivation_path: "N/A (from private key)".to_string(),
        })
    }

    fn validate_address(&self, address: &str) -> bool {
        // Use cosmrs AccountId parsing for proper Cosmos address validation
        AccountId::from_str(address).is_ok()
    }

    fn get_blockchain_name(&self) -> &'static str {
        "Cosmos"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosmos_address_validation() {
        let handler = CosmosHandler::new();

        // Generate a real Cosmos address to test validation
        let private_key = "1e99423a4ed27608a15a2616a2b0e9e52ced330ac530edcc32c8ffc6a526aedd";
        let result = handler.derive_from_private_key(private_key);
        assert!(result.is_ok());
        let wallet_keys = result.unwrap();

        // Test with the generated address
        assert!(handler.validate_address(&wallet_keys.address));

        // Test with known valid Cosmos addresses (these should be real valid addresses)
        // Note: Using real cosmos addresses for testing

        // Test invalid addresses
        assert!(!handler.validate_address("invalid_address"));
        assert!(!handler.validate_address("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2")); // Bitcoin address
        assert!(!handler.validate_address("0x742d35Cc6634C0532925a3b8D432C3475CB978B3")); // Ethereum address
        assert!(!handler.validate_address("")); // Empty string
        assert!(!handler.validate_address("cosmos")); // Too short
    }

    #[test]
    fn test_cosmos_private_key_derivation() {
        let handler = CosmosHandler::new();

        // Test with a known private key
        let private_key = "1e99423a4ed27608a15a2616a2b0e9e52ced330ac530edcc32c8ffc6a526aedd";
        let result = handler.derive_from_private_key(private_key);

        assert!(result.is_ok());
        let wallet_keys = result.unwrap();
        println!("Generated Cosmos address: {}", wallet_keys.address);

        // Verify the address format (Bech32 with 'cosmos' prefix)
        assert!(wallet_keys.address.starts_with("cosmos1")); // Should generate cosmos address
        assert!(!wallet_keys.address.is_empty());

        // Verify key sizes
        assert_eq!(wallet_keys.private_key.len(), 64); // 32 bytes as hex
        assert_eq!(wallet_keys.public_key.len(), 66); // 33 bytes compressed as hex

        // Validate the generated address passes our validation
        assert!(handler.validate_address(&wallet_keys.address));
    }

    #[test]
    fn test_cosmos_uses_correct_derivation_path() {
        let handler = CosmosHandler::new();

        // Test that Cosmos uses coin type 118
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let result = handler.derive_from_mnemonic(mnemonic, None, 0, 0, None);

        assert!(result.is_ok());
        let wallet_keys = result.unwrap();

        // Verify it uses Cosmos coin type (118)
        assert_eq!(wallet_keys.derivation_path, "m/44'/118'/0'/0/0");
        println!("Cosmos derivation path: {}", wallet_keys.derivation_path);
    }
}