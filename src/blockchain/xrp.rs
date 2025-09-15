use anyhow::{Result, Context};
use crate::blockchain::{BlockchainHandler, WalletKeys, SupportedBlockchain};
use crate::crypto::bip32::{derive_secp256k1_key_from_mnemonic, private_key_to_public_key_secp256k1};
use xrpl::core::addresscodec::encode_classic_address;

pub struct XrpHandler;

impl XrpHandler {
    pub fn new() -> Self {
        Self
    }

    fn public_key_to_address(&self, public_key_bytes: &[u8]) -> Result<String> {
        // Use xrpl library's address codec to create a classic address from public key
        // XRP addresses are created by hashing the public key and encoding it
        use sha2::{Sha256, Digest};
        use ripemd::{Ripemd160};

        // First hash with SHA256
        let mut sha_hasher = Sha256::new();
        sha_hasher.update(public_key_bytes);
        let sha_result = sha_hasher.finalize();

        // Then hash with RIPEMD160
        let mut ripemd_hasher = Ripemd160::new();
        ripemd_hasher.update(&sha_result);
        let account_id = ripemd_hasher.finalize();

        // Use xrpl library to encode the account ID as a classic address
        encode_classic_address(&account_id)
            .map_err(|e| anyhow::anyhow!("Failed to encode XRP address: {:?}", e))
    }
}

impl BlockchainHandler for XrpHandler {
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
            None => SupportedBlockchain::XRP.get_default_derivation_path(account, address_index),
        };

        // For now, derive the private key using our BIP-32 implementation
        // then create the XRP wallet from that private key
        let (private_key_bytes, public_key_bytes) = derive_secp256k1_key_from_mnemonic(
            mnemonic,
            passphrase,
            &derivation_path,
        )?;

        // Generate XRP address using xrpl library's address codec
        let address = self.public_key_to_address(&public_key_bytes)?;

        Ok(WalletKeys::new_simple(
            hex::encode(&private_key_bytes),
            hex::encode(&public_key_bytes),
            address,
            derivation_path,
        ))
    }

    fn derive_from_private_key(&self, private_key: &str) -> Result<WalletKeys> {
        // Parse private key from hex
        let private_key_bytes = hex::decode(private_key)
            .context("Invalid hex private key")?;

        // Derive public key from private key using our crypto utils
        let public_key_bytes = private_key_to_public_key_secp256k1(&private_key_bytes)?;

        // Generate XRP address using xrpl library's address codec
        let address = self.public_key_to_address(&public_key_bytes)?;

        Ok(WalletKeys::new_simple(
            private_key.to_string(),
            hex::encode(&public_key_bytes),
            address,
            "N/A (from private key)".to_string(),
        ))
    }

    fn validate_address(&self, address: &str) -> bool {
        // Use xrpl library's address validation functions
        use xrpl::core::addresscodec::is_valid_classic_address;

        is_valid_classic_address(address)
    }

    fn get_blockchain_name(&self) -> &'static str {
        "XRP"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xrp_address_validation() {
        let handler = XrpHandler::new();

        // Test with a known valid address that we can generate
        let private_key = "1e99423a4ed27608a15a2616a2b0e9e52ced330ac530edcc32c8ffc6a526aedd";
        let wallet_keys = handler.derive_from_private_key(private_key).unwrap();

        // Test that our generated address validates correctly
        assert!(handler.validate_address(&wallet_keys.address));

        // Test some known valid XRP addresses (these should be real mainnet addresses)
        assert!(handler.validate_address("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")); // Known valid address

        // Test invalid addresses
        assert!(!handler.validate_address("invalid_address"));
        assert!(!handler.validate_address("rInvalidAddress"));
        assert!(!handler.validate_address("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2")); // Bitcoin address
        assert!(!handler.validate_address("")); // Empty string
        assert!(!handler.validate_address("r")); // Too short
    }

    #[test]
    fn test_xrp_private_key_derivation() {
        let handler = XrpHandler::new();

        // Test with a known private key
        let private_key = "1e99423a4ed27608a15a2616a2b0e9e52ced330ac530edcc32c8ffc6a526aedd";
        let result = handler.derive_from_private_key(private_key);

        assert!(result.is_ok());
        let wallet_keys = result.unwrap();
        println!("Generated XRP address: {}", wallet_keys.address);
        assert!(wallet_keys.address.starts_with('r'));
        assert!(wallet_keys.address.len() >= 25);
        assert!(wallet_keys.address.len() <= 34);
    }
}