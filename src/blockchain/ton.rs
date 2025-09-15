use anyhow::{Result, Context};
use crate::blockchain::{BlockchainHandler, WalletKeys};
use crate::crypto::ed25519_utils::{derive_ed25519_key_from_mnemonic, private_key_to_public_key_ed25519};
use tonlib_core::{TonAddress, TonHash};

pub struct TonHandler;

impl TonHandler {
    pub fn new() -> Self {
        Self
    }

    fn public_key_to_address(&self, public_key_bytes: &[u8]) -> Result<String> {
        if public_key_bytes.len() != 32 {
            anyhow::bail!("TON public key must be 32 bytes, got {}", public_key_bytes.len());
        }

        // Use tonlib-core for proper TON address generation
        // TON addresses are derived from public keys using specific TON algorithms

        // Create TonHash from public key bytes
        let hash_bytes: [u8; 32] = public_key_bytes.try_into()
            .context("Failed to convert public key to 32-byte array")?;
        let ton_hash = TonHash::from(hash_bytes);

        // Create TON address with workchain 0 (mainnet) and the hash
        let ton_address = TonAddress::new(0, ton_hash);

        // Convert to base64 URL format (standard TON address format)
        Ok(ton_address.to_base64_url())
    }

    fn validate_ton_address(&self, address: &str) -> bool {
        // Use tonlib-core for proper TON address validation
        // The library handles parsing and validation of TON address formats
        TonAddress::from_base64_url(address).is_ok()
    }

    fn derive_from_mnemonic_ton(&self, mnemonic: &str, passphrase: Option<&str>) -> Result<WalletKeys> {
        // TON uses a custom derivation approach, but we'll use SLIP-0010 as a fallback
        // since tonlib-core focuses more on address handling than mnemonic derivation
        let derivation_path = "m/44'/607'/0'/0'/0'"; // TON coin type 607

        let (private_key_bytes, public_key_bytes) = derive_ed25519_key_from_mnemonic(
            mnemonic,
            passphrase,
            derivation_path,
        )?;

        let address = self.public_key_to_address(&public_key_bytes)?;

        Ok(WalletKeys::new_simple(
            hex::encode(&private_key_bytes),
            hex::encode(&public_key_bytes),
            address,
            derivation_path.to_string(),
        ))
    }
}

impl BlockchainHandler for TonHandler {
    fn derive_from_mnemonic(
        &self,
        mnemonic: &str,
        passphrase: Option<&str>,
        account: u32,
        address_index: u32,
        custom_path: Option<&str>,
    ) -> Result<WalletKeys> {
        // TON uses custom derivation, but we'll provide a fallback approach
        if let Some(path) = custom_path {
            // Use custom path if provided
            let (private_key_bytes, public_key_bytes) = derive_ed25519_key_from_mnemonic(
                mnemonic,
                passphrase,
                path,
            )?;

            let address = self.public_key_to_address(&public_key_bytes)?;

            Ok(WalletKeys::new_simple(
                hex::encode(&private_key_bytes),
                hex::encode(&public_key_bytes),
                address,
                path.to_string(),
            ))
        } else {
            // Use TON-specific derivation
            self.derive_from_mnemonic_ton(mnemonic, passphrase)
        }
    }

    fn derive_from_private_key(&self, private_key_hex: &str) -> Result<WalletKeys> {
        // Parse private key from hex
        let private_key_bytes = hex::decode(private_key_hex)
            .context("Invalid hex private key")?;

        if private_key_bytes.len() != 32 {
            anyhow::bail!("TON private key must be 32 bytes");
        }

        // Derive public key from private key
        let public_key_bytes = private_key_to_public_key_ed25519(&private_key_bytes)?;

        // Generate TON address from public key
        let address = self.public_key_to_address(&public_key_bytes)?;

        Ok(WalletKeys::new_simple(
            hex::encode(&private_key_bytes),
            hex::encode(&public_key_bytes),
            address,
            "Imported from private key".to_string(),
        ))
    }

    fn validate_address(&self, address: &str) -> bool {
        // Use TON-specific address validation
        self.validate_ton_address(address)
    }

    fn get_blockchain_name(&self) -> &'static str {
        "TON"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ton_address_validation() {
        let handler = TonHandler::new();

        // Generate a real TON address to test validation
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
        assert!(!handler.validate_address("ton")); // Too short
    }

    #[test]
    fn test_ton_private_key_derivation() {
        let handler = TonHandler::new();
        let private_key = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

        let result = handler.derive_from_private_key(private_key);
        assert!(result.is_ok());

        let keys = result.unwrap();
        assert_eq!(keys.private_key, private_key);
        assert!(keys.public_key.len() == 64); // 32 bytes in hex
        assert!(handler.validate_address(&keys.address));

        // Print the generated address for debugging
        println!("Generated TON address: {}", keys.address);

        // Verify TON address format (EQ/UQ prefix and appropriate length)
        assert!(keys.address.starts_with("EQ") || keys.address.starts_with("UQ"));
        assert!(keys.address.len() >= 40);
    }
}