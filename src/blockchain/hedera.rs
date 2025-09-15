use anyhow::{Result, Context};
use crate::blockchain::{BlockchainHandler, WalletKeys};
use crate::crypto::ed25519_utils::{derive_ed25519_key_from_mnemonic, private_key_to_public_key_ed25519};
use crate::crypto::bip32::{derive_secp256k1_key_from_mnemonic, private_key_to_public_key_secp256k1};
use hiero_sdk::{PrivateKey as HederaPrivateKey, PublicKey as HederaPublicKey, AccountId as HederaAccountId, Client};
use std::collections::HashMap;

pub struct HederaHandler;

impl HederaHandler {
    pub fn new() -> Self {
        Self
    }

    // Enhanced address generation supporting both ED25519 and ECDSA
    fn generate_hedera_addresses(&self, public_key_bytes: &[u8], use_ecdsa: bool) -> Result<(String, Option<String>, HashMap<String, String>, HashMap<String, String>)> {
        let mut additional_data = HashMap::new();
        let mut secondary_addresses = HashMap::new();

        let (hedera_public_key, account_id, address) = if use_ecdsa {
            // ECDSA implementation for EVM compatibility
            let hedera_public_key = HederaPublicKey::from_bytes_ecdsa(public_key_bytes)
                .map_err(|e| anyhow::anyhow!("Failed to create Hedera ECDSA public key: {}", e))?;

            let account_id = hedera_public_key.to_account_id(0, 0);
            let address = account_id.to_string();

            // Generate EVM address if ECDSA
            if let Some(evm_address) = hedera_public_key.to_evm_address() {
                let evm_addr_str = format!("0x{:x}", evm_address);
                secondary_addresses.insert("evm".to_string(), evm_addr_str);
            }

            additional_data.insert("key_type".to_string(), "ecdsa".to_string());
            (hedera_public_key, account_id, address)
        } else {
            // ED25519 implementation (existing)
            let hedera_public_key = HederaPublicKey::from_bytes_ed25519(public_key_bytes)
                .map_err(|e| anyhow::anyhow!("Failed to create Hedera ED25519 public key: {}", e))?;

            let account_id = hedera_public_key.to_account_id(0, 0);
            let address = account_id.to_string();

            additional_data.insert("key_type".to_string(), "ed25519".to_string());
            (hedera_public_key, account_id, address)
        };

        // Generate checksummed address
        // Note: This would require a client context in real usage. For now, we'll create a placeholder
        // In a real implementation, you'd need: account_id.to_string_with_checksum(&client)
        let address_with_checksum = Some(format!("{}-placeholder_checksum", address));

        additional_data.insert("shard".to_string(), "0".to_string());
        additional_data.insert("realm".to_string(), "0".to_string());

        Ok((address, address_with_checksum, additional_data, secondary_addresses))
    }

    // Legacy method for backward compatibility
    fn public_key_to_address(&self, public_key_bytes: &[u8]) -> Result<String> {
        let (address, _, _, _) = self.generate_hedera_addresses(public_key_bytes, false)?;
        Ok(address)
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

        // For demonstration, we'll default to ED25519 but could add ECDSA option
        // In a full implementation, you might want to make this configurable
        let use_ecdsa = false; // Could be made configurable via custom_path or additional parameter

        let (private_key_bytes, public_key_bytes) = if use_ecdsa {
            // Derive secp256k1 private and public keys for ECDSA/EVM compatibility
            derive_secp256k1_key_from_mnemonic(
                mnemonic,
                passphrase,
                &derivation_path,
            )?
        } else {
            // Derive ed25519 private and public keys using SLIP-0010
            derive_ed25519_key_from_mnemonic(
                mnemonic,
                passphrase,
                &derivation_path,
            )?
        };

        // Generate Hedera addresses with metadata
        let (address, address_with_checksum, additional_data, secondary_addresses) =
            self.generate_hedera_addresses(&public_key_bytes, use_ecdsa)?;

        let mut wallet_keys = WalletKeys::new_with_checksum(
            hex::encode(&private_key_bytes),
            hex::encode(&public_key_bytes),
            address,
            address_with_checksum,
            derivation_path,
        );

        // Add additional data and secondary addresses
        for (key, value) in additional_data {
            wallet_keys.add_data(key, value);
        }
        for (addr_type, addr) in secondary_addresses {
            wallet_keys.add_secondary_address(addr_type, addr);
        }

        Ok(wallet_keys)
    }

    fn derive_from_private_key(&self, private_key_hex: &str) -> Result<WalletKeys> {
        // Parse private key from hex
        let private_key_bytes = hex::decode(private_key_hex)
            .context("Invalid hex private key")?;

        if private_key_bytes.len() != 32 {
            anyhow::bail!("Hedera private key must be 32 bytes");
        }

        // For now, assume ED25519 for imported keys. Could be enhanced to detect key type
        let use_ecdsa = false;

        let public_key_bytes = if use_ecdsa {
            private_key_to_public_key_secp256k1(&private_key_bytes)?
        } else {
            private_key_to_public_key_ed25519(&private_key_bytes)?
        };

        // Generate Hedera addresses with metadata
        let (address, address_with_checksum, additional_data, secondary_addresses) =
            self.generate_hedera_addresses(&public_key_bytes, use_ecdsa)?;

        let mut wallet_keys = WalletKeys::new_with_checksum(
            hex::encode(&private_key_bytes),
            hex::encode(&public_key_bytes),
            address,
            address_with_checksum,
            "Imported from private key".to_string(),
        );

        // Add additional data and secondary addresses
        for (key, value) in additional_data {
            wallet_keys.add_data(key, value);
        }
        for (addr_type, addr) in secondary_addresses {
            wallet_keys.add_secondary_address(addr_type, addr);
        }

        Ok(wallet_keys)
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