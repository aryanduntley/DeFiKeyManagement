use anyhow::{Result, Context};
use crate::blockchain::{BlockchainHandler, WalletKeys};
use crate::crypto::ed25519_utils::{derive_ed25519_key_from_mnemonic, private_key_to_public_key_ed25519};
use iota_sdk::types::block::address::{Address, Ed25519Address, Bech32Address};
use iota_sdk::client::constants::IOTA_BECH32_HRP;

pub struct IotaHandler;

impl IotaHandler {
    pub fn new() -> Self {
        Self
    }

    fn public_key_to_address(&self, public_key_bytes: &[u8]) -> Result<String> {
        if public_key_bytes.len() != 32 {
            anyhow::bail!("IOTA public key must be 32 bytes, got {}", public_key_bytes.len());
        }

        // Use official IOTA SDK for address generation
        // Create Ed25519Address from public key bytes
        let ed25519_address = Ed25519Address::new(
            public_key_bytes.try_into()
                .context("Failed to convert public key to 32-byte array")?
        );

        // Convert to Address enum
        let address = Address::Ed25519(ed25519_address);

        // Create Bech32Address with IOTA mainnet HRP
        let bech32_address = Bech32Address::new(IOTA_BECH32_HRP, address);

        // Convert to string
        Ok(bech32_address.to_string())
    }

    fn validate_iota_address(&self, address: &str) -> bool {
        // Use official IOTA SDK address validation
        // Try to parse as Bech32Address and check if it's valid IOTA address
        if let Ok(bech32_addr) = address.parse::<Bech32Address>() {
            *bech32_addr.hrp() == IOTA_BECH32_HRP && matches!(bech32_addr.inner(), Address::Ed25519(_))
        } else {
            false
        }
    }
}

impl BlockchainHandler for IotaHandler {
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
                // IOTA uses SLIP-0010 derivation path: m/44'/4218'/0'/0'/0'
                format!("m/44'/4218'/{}'/{}'/{}'", account, 0, address_index)
            },
        };

        // Derive ed25519 private and public keys using SLIP-0010
        let (private_key_bytes, public_key_bytes) = derive_ed25519_key_from_mnemonic(
            mnemonic,
            passphrase,
            &derivation_path,
        )?;

        // Generate IOTA address from public key
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
            anyhow::bail!("IOTA private key must be 32 bytes");
        }

        // Derive public key from private key
        let public_key_bytes = private_key_to_public_key_ed25519(&private_key_bytes)?;

        // Generate IOTA address from public key
        let address = self.public_key_to_address(&public_key_bytes)?;

        Ok(WalletKeys::new_simple(
            hex::encode(&private_key_bytes),
            hex::encode(&public_key_bytes),
            address,
            "Imported from private key".to_string(),
        ))
    }

    fn validate_address(&self, address: &str) -> bool {
        // Use IOTA-specific Bech32 address validation
        self.validate_iota_address(address)
    }

    fn get_blockchain_name(&self) -> &'static str {
        "IOTA"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iota_address_validation() {
        let handler = IotaHandler::new();

        // Generate a real IOTA address to test validation
        let private_key = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let result = handler.derive_from_private_key(private_key);
        assert!(result.is_ok());
        let wallet_keys = result.unwrap();

        // Test with the generated address
        assert!(handler.validate_address(&wallet_keys.address));

        // Test with known invalid addresses
        assert!(!handler.validate_address("invalid_address"));
        assert!(!handler.validate_address("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2")); // Bitcoin address
        assert!(!handler.validate_address("0x742d35Cc6634C0532925a3b8D432C3475CB978B3")); // Ethereum address
        assert!(!handler.validate_address("")); // Empty string
        assert!(!handler.validate_address("iota")); // Too short
    }

    #[test]
    fn test_iota_private_key_derivation() {
        let handler = IotaHandler::new();
        let private_key = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

        let result = handler.derive_from_private_key(private_key);
        assert!(result.is_ok());

        let keys = result.unwrap();
        assert_eq!(keys.private_key, private_key);
        assert!(keys.public_key.len() == 64); // 32 bytes in hex
        assert!(handler.validate_address(&keys.address));

        // Print the generated address for debugging
        println!("Generated IOTA address: {}", keys.address);

        // Verify IOTA address format (iota1 prefix and appropriate length)
        assert!(keys.address.starts_with("iota1"));
        assert!(keys.address.len() >= 40);
    }
}