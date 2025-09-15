use anyhow::{Result, Context};
use crate::blockchain::{BlockchainHandler, WalletKeys, SupportedBlockchain};
use crate::crypto::bip32::{derive_secp256k1_key_from_mnemonic, private_key_to_public_key_secp256k1};
use anychain_tron::{TronAddress, TronPublicKey, TronFormat};
use anychain_core::PublicKey;
use std::str::FromStr;

pub struct TronHandler;

impl TronHandler {
    pub fn new() -> Self {
        Self
    }

    fn public_key_to_address(&self, public_key_bytes: &[u8]) -> Result<String> {
        // Use anychain-tron library for proper TRON address generation

        // Convert public key bytes to TronPublicKey using hex encoding
        let public_key_hex = hex::encode(public_key_bytes);
        let tron_public_key = TronPublicKey::from_str(&public_key_hex)
            .map_err(|e| anyhow::anyhow!("Failed to create TronPublicKey: {}", e))?;

        // Use the direct to_address method with Standard format
        let format = TronFormat::Standard;
        let tron_address = tron_public_key.to_address(&format)
            .map_err(|e| anyhow::anyhow!("Failed to generate TRON address: {}", e))?;

        let address_string = tron_address.to_string();
        println!("Generated TRON address: {}", address_string);

        Ok(address_string)
    }
}

impl BlockchainHandler for TronHandler {
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
            None => SupportedBlockchain::Tron.get_default_derivation_path(account, address_index),
        };

        // Derive private and public key using BIP-32 (secp256k1 curve)
        let (private_key_bytes, public_key_bytes) = derive_secp256k1_key_from_mnemonic(
            mnemonic,
            passphrase,
            &derivation_path,
        )?;

        // Generate TRON address from public key
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
            return Err(anyhow::anyhow!("TRON private key must be 32 bytes"));
        }

        // Derive public key
        let public_key_bytes = private_key_to_public_key_secp256k1(&private_key_bytes)?;

        // Generate TRON address
        let address = self.public_key_to_address(&public_key_bytes)?;

        Ok(WalletKeys {
            address,
            public_key: hex::encode(&public_key_bytes),
            private_key: hex::encode(&private_key_bytes),
            derivation_path: "N/A (from private key)".to_string(),
        })
    }

    fn validate_address(&self, address: &str) -> bool {
        // Use anychain-tron library for proper TRON address validation
        match TronAddress::from_str(address) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    fn get_blockchain_name(&self) -> &'static str {
        "TRON"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tron_address_validation() {
        let handler = TronHandler::new();

        // Test valid TRON addresses using our generated address
        let generated_address = "TYHwcdSLEBSXasK9xp6JDPieYLosL8239x";
        assert!(handler.validate_address(generated_address));

        // Test with another generated address
        let private_key = "2e99423a4ed27608a15a2616a2b0e9e52ced330ac530edcc32c8ffc6a526aedd";
        if let Ok(wallet_keys) = handler.derive_from_private_key(private_key) {
            assert!(handler.validate_address(&wallet_keys.address));
            println!("Validating generated TRON address: {}", wallet_keys.address);
        }

        // Test invalid addresses
        assert!(!handler.validate_address("invalid_address"));
        assert!(!handler.validate_address("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2")); // Bitcoin address
        assert!(!handler.validate_address("0x742d35Cc6634C0532925a3b8D432C3475CB978B3")); // Ethereum address
        assert!(!handler.validate_address("")); // Empty string
        assert!(!handler.validate_address("T")); // Too short
    }

    #[test]
    fn test_tron_private_key_derivation() {
        let handler = TronHandler::new();

        // Test with a known private key
        let private_key = "1e99423a4ed27608a15a2616a2b0e9e52ced330ac530edcc32c8ffc6a526aedd";
        let result = handler.derive_from_private_key(private_key);

        assert!(result.is_ok());
        let wallet_keys = result.unwrap();
        println!("Generated TRON address: {}", wallet_keys.address);
        assert!(wallet_keys.address.starts_with("T")); // Should generate TRON address
        assert!(!wallet_keys.address.is_empty());
        assert_eq!(wallet_keys.private_key.len(), 64); // 32 bytes as hex
        assert_eq!(wallet_keys.public_key.len(), 66); // 33 bytes compressed as hex
    }
}