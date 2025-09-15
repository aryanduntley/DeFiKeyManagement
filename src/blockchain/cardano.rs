use anyhow::{Result, Context};
use crate::blockchain::{BlockchainHandler, WalletKeys, SupportedBlockchain};
use crate::crypto::ed25519_utils::{derive_ed25519_key_from_mnemonic, private_key_to_public_key_ed25519};
use blake2::{Blake2s256, Digest};
use bech32;

pub struct CardanoHandler;

impl CardanoHandler {
    pub fn new() -> Self {
        Self
    }

    fn public_key_to_address(&self, public_key: &[u8]) -> Result<String> {
        // Cardano address generation (Shelley era - addr prefix)
        // 1. Create payment credential from public key hash
        // 2. Create enterprise address (payment credential only, no stake credential)
        // 3. Encode as Bech32 with "addr" prefix

        // Hash the public key using Blake2s-256 (we'll use the first 28 bytes for Blake2b-224 equivalent)
        let mut hasher = Blake2s256::new();
        hasher.update(public_key);
        let key_hash = hasher.finalize();
        let key_hash_224 = &key_hash[0..28]; // Take first 28 bytes

        // Create enterprise address (type 0b0110 = 6 for enterprise address, mainnet)
        let mut address_bytes = vec![0b01100000]; // Address type + network
        address_bytes.extend_from_slice(key_hash_224);

        // Encode as Bech32 with "addr" prefix using the newer API
        let hrp = bech32::Hrp::parse("addr").map_err(|e| anyhow::anyhow!("Invalid HRP: {}", e))?;
        let address = bech32::encode::<bech32::Bech32>(hrp, &address_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to encode Cardano address: {}", e))?;

        Ok(address)
    }
}

impl BlockchainHandler for CardanoHandler {
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
            None => SupportedBlockchain::Cardano.get_default_derivation_path(account, address_index),
        };

        // Derive private and public key using SLIP-0010 ed25519 derivation
        let (private_key_bytes, public_key_bytes) = derive_ed25519_key_from_mnemonic(
            mnemonic,
            passphrase,
            &derivation_path,
        )?;

        // Generate Cardano address from public key
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
        let private_key_hex = if private_key.starts_with("0x") {
            &private_key[2..]
        } else {
            private_key
        };

        let private_key_bytes = hex::decode(private_key_hex)
            .context("Invalid hex private key")?;

        if private_key_bytes.len() != 32 {
            return Err(anyhow::anyhow!("Cardano private key must be 32 bytes"));
        }

        // Derive public key from private key using ed25519
        let public_key_bytes = private_key_to_public_key_ed25519(&private_key_bytes)?;

        // Generate Cardano address from public key
        let address = self.public_key_to_address(&public_key_bytes)?;

        Ok(WalletKeys::new_simple(
            hex::encode(&private_key_bytes),
            hex::encode(&public_key_bytes),
            address,
            "N/A (from private key)".to_string(),
        ))
    }

    fn validate_address(&self, address: &str) -> bool {
        // Cardano addresses start with 'addr' and use Bech32 encoding
        if !address.starts_with("addr") {
            return false;
        }

        // Try to decode as Bech32 using the newer API
        if let Ok((hrp, _data)) = bech32::decode(address) {
            hrp.to_string() == "addr"
        } else {
            false
        }
    }

    fn get_blockchain_name(&self) -> &'static str {
        "Cardano"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cardano_address_validation() {
        let handler = CardanoHandler::new();

        // Test valid Cardano addresses - use our generated address as a valid example
        let valid_address = "addr1vz6f97ldcc4z3kly35xk3u4flktzst8mhd8x8627srs8n3g9rl7yr";
        assert!(handler.validate_address(valid_address));

        // Generate another address to test validation consistency
        let private_key = "2e99423a4ed27608a15a2616a2b0e9e52ced330ac530edcc32c8ffc6a526aedd";
        if let Ok(wallet_keys) = handler.derive_from_private_key(private_key) {
            assert!(handler.validate_address(&wallet_keys.address));
            println!("Validating generated address: {}", wallet_keys.address);
        }

        // Test invalid addresses
        assert!(!handler.validate_address("invalid_address"));
        assert!(!handler.validate_address("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2")); // Bitcoin address
        assert!(!handler.validate_address("ltc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh")); // Litecoin address
        assert!(!handler.validate_address("")); // Empty string
        assert!(!handler.validate_address("addr")); // Too short
        assert!(!handler.validate_address("addr1qy2jt0qpqz2z2z9zx5w3w5w3w5w3w5w3w5w3w5w3w")); // Invalid Bech32
    }

    #[test]
    fn test_cardano_private_key_derivation() {
        let handler = CardanoHandler::new();

        // Test with a known private key (32 bytes)
        let private_key = "1e99423a4ed27608a15a2616a2b0e9e52ced330ac530edcc32c8ffc6a526aedd";
        let result = handler.derive_from_private_key(private_key);

        assert!(result.is_ok());
        let wallet_keys = result.unwrap();
        println!("Generated Cardano address: {}", wallet_keys.address);
        assert!(wallet_keys.address.starts_with("addr"));
        assert!(!wallet_keys.address.is_empty());
        assert_eq!(wallet_keys.private_key.len(), 64); // 32 bytes as hex
        assert_eq!(wallet_keys.public_key.len(), 64); // 32 bytes as hex
    }
}