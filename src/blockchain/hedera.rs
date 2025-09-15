use anyhow::{Result, Context};
use crate::blockchain::{BlockchainHandler, WalletKeys};
use crate::crypto::ed25519_utils::{derive_ed25519_key_from_mnemonic, private_key_to_public_key_ed25519};
use hiero_sdk::{PrivateKey as HederaPrivateKey, PublicKey as HederaPublicKey, AccountId as HederaAccountId};

pub struct HederaHandler;

impl HederaHandler {
    pub fn new() -> Self {
        Self
    }

    fn public_key_to_address(&self, public_key_bytes: &[u8]) -> Result<String> {
        // Hedera address generation using hiero-sdk:
        // 1. Create HederaPublicKey from raw ed25519 public key bytes
        // 2. Generate AccountId with shard=0, realm=0 and public key as alias
        // 3. Convert to string representation

        // Create Hedera PublicKey from raw ed25519 bytes
        let hedera_public_key = HederaPublicKey::from_bytes_ed25519(public_key_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to create Hedera public key: {}", e))?;

        // Generate AccountId with the public key as an alias (shard=0, realm=0)
        let account_id = hedera_public_key.to_account_id(0, 0);

        // Convert to string representation
        Ok(account_id.to_string())
    }
}

impl BlockchainHandler for HederaHandler {
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
                // Hedera uses SLIP-0010 derivation path: m/44'/3030'/0'/0'/0'
                format!("m/44'/3030'/{}'/{}'/{}'", account, 0, address_index)
            },
        };

        // Derive ed25519 private and public keys using SLIP-0010
        let (private_key_bytes, public_key_bytes) = derive_ed25519_key_from_mnemonic(
            mnemonic,
            passphrase,
            &derivation_path,
        )?;

        // Generate Hedera address from public key
        let address = self.public_key_to_address(&public_key_bytes)?;

        Ok(WalletKeys {
            private_key: hex::encode(&private_key_bytes),
            public_key: hex::encode(&public_key_bytes),
            address,
            derivation_path,
        })
    }

    fn derive_from_private_key(&self, private_key_hex: &str) -> Result<WalletKeys> {
        // Parse private key from hex
        let private_key_bytes = hex::decode(private_key_hex)
            .context("Invalid hex private key")?;

        if private_key_bytes.len() != 32 {
            anyhow::bail!("Hedera private key must be 32 bytes");
        }

        // Derive public key from private key
        let public_key_bytes = private_key_to_public_key_ed25519(&private_key_bytes)?;

        // Generate Hedera address from public key
        let address = self.public_key_to_address(&public_key_bytes)?;

        Ok(WalletKeys {
            private_key: hex::encode(&private_key_bytes),
            public_key: hex::encode(&public_key_bytes),
            address,
            derivation_path: "Imported from private key".to_string(),
        })
    }

    fn validate_address(&self, address: &str) -> bool {
        // For our wallet tool, we only accept traditional Hedera format (shard.realm.account)
        // Even though Hedera supports EVM addresses, we treat those as Ethereum blockchain addresses

        // Reject EVM/Ethereum format addresses (0x...)
        if address.starts_with("0x") || address.starts_with("0X") {
            return false;
        }

        // Hedera addresses must contain dots for shard.realm.account format
        if !address.contains('.') {
            return false;
        }

        // Use hiero-sdk AccountId parsing for proper validation
        match address.parse::<HederaAccountId>() {
            Ok(_account_id) => {
                // Additional validation: ensure it's traditional Hedera format (shard.realm.account)
                let parts: Vec<&str> = address.split('.').collect();
                if parts.len() == 3 {
                    // All parts must be non-empty and the first two should be numbers (shard.realm)
                    parts.iter().all(|part| !part.is_empty()) &&
                    parts[0].chars().all(|c| c.is_ascii_digit()) &&
                    parts[1].chars().all(|c| c.is_ascii_digit())
                    // Third part (account) can be numeric or hex (public key alias)
                } else {
                    false
                }
            },
            Err(_) => false
        }
    }

    fn get_blockchain_name(&self) -> &'static str {
        "Hedera"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hedera_address_validation() {
        let handler = HederaHandler::new();

        // Generate a real Hedera address to test validation
        let private_key = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let result = handler.derive_from_private_key(private_key);
        assert!(result.is_ok());
        let wallet_keys = result.unwrap();

        // Test with the generated address
        assert!(handler.validate_address(&wallet_keys.address));

        // Test with known valid Hedera addresses (numeric format)
        assert!(handler.validate_address("0.0.123456"));
        assert!(handler.validate_address("0.0.1"));
        assert!(handler.validate_address("1.2.999999"));

        // Invalid addresses
        assert!(!handler.validate_address("0.0"));
        assert!(!handler.validate_address("0.0.123.456"));
        assert!(!handler.validate_address("invalid"));
        assert!(!handler.validate_address("0.0.abc"));
        assert!(!handler.validate_address("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2")); // Bitcoin address
        assert!(!handler.validate_address("0x742d35Cc6634C0532925a3b8D432C3475CB978B3")); // Ethereum address
        assert!(!handler.validate_address("")); // Empty string
    }

    #[test]
    fn test_hedera_private_key_derivation() {
        let handler = HederaHandler::new();
        let private_key = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

        let result = handler.derive_from_private_key(private_key);
        assert!(result.is_ok());

        let keys = result.unwrap();
        assert_eq!(keys.private_key, private_key);
        assert!(keys.public_key.len() == 64); // 32 bytes in hex
        assert!(handler.validate_address(&keys.address));

        // Print the generated address for debugging
        println!("Generated Hedera address: {}", keys.address);

        // Verify this is a Hedera-style address (should be long hex string due to public key alias)
        // Hedera addresses with public key aliases are much longer than simple 0.0.xxxx format
        assert!(keys.address.len() > 20); // Much longer than simple numeric format
        assert!(keys.address.starts_with("0.0.")); // Should start with shard.realm
    }
}