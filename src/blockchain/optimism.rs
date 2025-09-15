use anyhow::{Result, Context};
use crate::blockchain::{BlockchainHandler, WalletKeys};
use crate::crypto::bip32::{derive_secp256k1_key_from_mnemonic, private_key_to_public_key_secp256k1};
use alloy_primitives::{Address, keccak256};
use k256::ecdsa::VerifyingKey;
use k256::elliptic_curve::sec1::ToEncodedPoint;

pub struct OptimismHandler;

impl OptimismHandler {
    pub fn new() -> Self {
        Self
    }

    fn public_key_to_address(&self, public_key_bytes: &[u8]) -> Result<String> {
        // Optimism uses identical address generation to Ethereum:
        // 1. Take uncompressed public key (64 bytes, remove 0x04 prefix)
        // 2. Compute Keccak256 hash
        // 3. Take last 20 bytes
        // 4. Apply EIP-55 checksum using alloy-primitives

        // Convert compressed public key to uncompressed format
        let verifying_key = VerifyingKey::from_sec1_bytes(public_key_bytes)
            .context("Invalid public key for Optimism address generation")?;

        let uncompressed_point = verifying_key.to_encoded_point(false);
        let uncompressed_bytes = uncompressed_point.as_bytes();

        // Remove the 0x04 prefix (first byte) to get the 64-byte coordinate pair
        let public_key_hash_input = &uncompressed_bytes[1..];

        // Compute Keccak256 hash using alloy-primitives
        let hash = keccak256(public_key_hash_input);

        // Take the last 20 bytes as the address
        let address_bytes: [u8; 20] = hash[12..].try_into()
            .context("Failed to extract address bytes from hash")?;

        // Create Address using alloy-primitives (automatically applies EIP-55 checksum)
        let address = Address::from(address_bytes);

        Ok(address.to_string())
    }

}

impl BlockchainHandler for OptimismHandler {
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
                // Optimism uses Ethereum's derivation path: m/44'/60'/0'/0/0
                // This is different from most other chains that have their own coin type
                format!("m/44'/60'/{}'/{}/{}", account, 0, address_index)
            },
        };

        // Derive private and public key using BIP-32 (secp256k1 curve)
        let (private_key_bytes, public_key_bytes) = derive_secp256k1_key_from_mnemonic(
            mnemonic,
            passphrase,
            &derivation_path,
        )?;

        // Generate Optimism address from public key (ETH-identical)
        let address = self.public_key_to_address(&public_key_bytes)?;

        Ok(WalletKeys::new_simple(
            hex::encode(&private_key_bytes),
            hex::encode(&public_key_bytes),
            address,
            derivation_path,
        ))
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
            return Err(anyhow::anyhow!("Optimism private key must be 32 bytes"));
        }

        // Derive public key
        let public_key_bytes = private_key_to_public_key_secp256k1(&private_key_bytes)?;

        // Generate Optimism address
        let address = self.public_key_to_address(&public_key_bytes)?;

        Ok(WalletKeys::new_simple(
            hex::encode(&private_key_bytes),
            hex::encode(&public_key_bytes),
            address,
            "N/A (from private key)".to_string(),
        ))
    }

    fn validate_address(&self, address: &str) -> bool {
        // Use alloy-primitives Address parsing for proper EIP-55 checksum validation
        Address::parse_checksummed(address, None).is_ok()
    }

    fn get_blockchain_name(&self) -> &'static str {
        "Optimism"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimism_address_validation() {
        let handler = OptimismHandler::new();

        // Generate a real Optimism address to test validation
        let private_key = "1e99423a4ed27608a15a2616a2b0e9e52ced330ac530edcc32c8ffc6a526aedd";
        let result = handler.derive_from_private_key(private_key);
        assert!(result.is_ok());
        let wallet_keys = result.unwrap();

        // Test with the generated address
        assert!(handler.validate_address(&wallet_keys.address));

        // Test with known valid Ethereum-compatible addresses with proper EIP-55 checksums
        assert!(handler.validate_address("0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed"));
        assert!(handler.validate_address("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")); // Proper checksum

        // Test invalid addresses
        assert!(!handler.validate_address("invalid_address"));
        assert!(!handler.validate_address("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2")); // Bitcoin address
        assert!(!handler.validate_address("TLyqzVGLV1srkB7dToTAEqgDSfPtXRJZYH")); // TRON address
        assert!(!handler.validate_address("")); // Empty string
        assert!(!handler.validate_address("0x")); // Too short
        assert!(!handler.validate_address("742d35Cc6634C0532925a3b8D432C3475CB978B3")); // Missing 0x prefix
        assert!(!handler.validate_address("0x742d35Cc6634C0532925a3b8D432C3475CB978G3")); // Invalid hex character
        assert!(!handler.validate_address("0x742d35cc6634c0532925a3b8d432c3475cb978b3")); // Invalid checksum (all lowercase)
    }

    #[test]
    fn test_optimism_private_key_derivation() {
        let handler = OptimismHandler::new();

        // Test with a known private key
        let private_key = "1e99423a4ed27608a15a2616a2b0e9e52ced330ac530edcc32c8ffc6a526aedd";
        let result = handler.derive_from_private_key(private_key);

        assert!(result.is_ok());
        let wallet_keys = result.unwrap();
        println!("Generated Optimism address: {}", wallet_keys.address);

        // Verify the address format (Ethereum-compatible with EIP-55 checksum)
        assert!(wallet_keys.address.starts_with("0x")); // Should generate Ethereum-style address
        assert_eq!(wallet_keys.address.len(), 42); // Standard Ethereum address length
        assert!(!wallet_keys.address.is_empty());

        // Verify the address has proper EIP-55 mixed case checksumming
        assert!(wallet_keys.address.chars().skip(2).any(|c| c.is_ascii_uppercase())); // Should have some uppercase
        assert!(wallet_keys.address.chars().skip(2).any(|c| c.is_ascii_lowercase())); // Should have some lowercase

        // Verify key sizes
        assert_eq!(wallet_keys.private_key.len(), 64); // 32 bytes as hex
        assert_eq!(wallet_keys.public_key.len(), 66); // 33 bytes compressed as hex

        // Validate the generated address passes our validation
        assert!(handler.validate_address(&wallet_keys.address));
    }

    #[test]
    fn test_optimism_uses_ethereum_derivation_path() {
        let handler = OptimismHandler::new();

        // Test that Optimism uses Ethereum's derivation path
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let result = handler.derive_from_mnemonic(mnemonic, None, 0, 0, None);

        assert!(result.is_ok());
        let wallet_keys = result.unwrap();

        // Verify it uses Ethereum's coin type (60) not a unique Optimism coin type
        assert_eq!(wallet_keys.derivation_path, "m/44'/60'/0'/0/0");
        println!("Optimism derivation path: {}", wallet_keys.derivation_path);
    }
}