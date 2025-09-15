use anyhow::{Result, Context};
use crate::blockchain::{BlockchainHandler, WalletKeys};
use crate::crypto::ed25519_utils::{derive_ed25519_key_from_mnemonic, private_key_to_public_key_ed25519};
use blake2::{Blake2b512, Digest};
use ss58_registry::Ss58AddressFormat;

pub struct PolkadotHandler;

impl PolkadotHandler {
    pub fn new() -> Self {
        Self
    }

    fn public_key_to_address(&self, public_key_bytes: &[u8]) -> Result<String> {
        if public_key_bytes.len() != 32 {
            anyhow::bail!("Polkadot public key must be 32 bytes, got {}", public_key_bytes.len());
        }

        // Polkadot SS58 address encoding:
        // 1. Concatenate: network_prefix + public_key
        // 2. Hash with Blake2b-512
        // 3. Take first 2 bytes as checksum
        // 4. Concatenate: network_prefix + public_key + checksum
        // 5. Encode with Base58

        let network_prefix = 0u8; // Polkadot mainnet prefix

        // Step 1: Create payload (network_prefix + public_key)
        let mut payload = Vec::new();
        payload.push(network_prefix);
        payload.extend_from_slice(public_key_bytes);

        // Step 2: Create checksum context for hashing
        let ss58_prefix = b"SS58PRE";
        let mut hasher = Blake2b512::new(); // Blake2b with 64-byte output
        hasher.update(ss58_prefix);
        hasher.update(&payload);
        let hash = hasher.finalize();

        // Step 3: Take first 2 bytes as checksum
        let checksum = &hash[..2];

        // Step 4: Create final payload (network_prefix + public_key + checksum)
        let mut final_payload = payload;
        final_payload.extend_from_slice(checksum);

        // Step 5: Encode with Base58
        let address = bs58::encode(&final_payload).into_string();

        Ok(address)
    }

    fn validate_ss58_address(&self, address: &str) -> bool {
        // Polkadot addresses are SS58-encoded with specific format:
        // 1. Should be valid Base58
        // 2. Should start with '1' for Polkadot mainnet (network prefix 0)
        // 3. Should be 47-48 characters long
        // 4. Checksum should be valid

        if address.len() < 35 || address.len() > 90 {
            return false;
        }

        // Decode Base58
        let decoded = match bs58::decode(address).into_vec() {
            Ok(bytes) => bytes,
            Err(_) => return false,
        };

        // Should have at least: 1 byte prefix + 32 bytes public key + 2 bytes checksum = 35 bytes
        if decoded.len() < 35 {
            return false;
        }

        // Extract components
        let network_prefix = decoded[0];
        let payload = &decoded[..decoded.len() - 2];
        let provided_checksum = &decoded[decoded.len() - 2..];

        // Validate network prefix (0 for Polkadot mainnet)
        if network_prefix != 0 {
            return false;
        }

        // Calculate expected checksum
        let ss58_prefix = b"SS58PRE";
        let mut hasher = Blake2b512::new(); // Blake2b with 64-byte output
        hasher.update(ss58_prefix);
        hasher.update(payload);
        let hash = hasher.finalize();
        let expected_checksum = &hash[..2];

        // Compare checksums
        provided_checksum == expected_checksum
    }
}

impl BlockchainHandler for PolkadotHandler {
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
                // Polkadot uses SLIP-0010 derivation path: m/44'/354'/0'/0'/0'
                format!("m/44'/354'/{}'/{}'/{}'", account, 0, address_index)
            },
        };

        // Derive ed25519 private and public keys using SLIP-0010
        let (private_key_bytes, public_key_bytes) = derive_ed25519_key_from_mnemonic(
            mnemonic,
            passphrase,
            &derivation_path,
        )?;

        // Generate Polkadot address from public key
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
            anyhow::bail!("Polkadot private key must be 32 bytes");
        }

        // Derive public key from private key
        let public_key_bytes = private_key_to_public_key_ed25519(&private_key_bytes)?;

        // Generate Polkadot address from public key
        let address = self.public_key_to_address(&public_key_bytes)?;

        Ok(WalletKeys::new_simple(
            hex::encode(&private_key_bytes),
            hex::encode(&public_key_bytes),
            address,
            "Imported from private key".to_string(),
        ))
    }

    fn validate_address(&self, address: &str) -> bool {
        // Custom SS58 address validation for Polkadot
        self.validate_ss58_address(address)
    }

    fn get_blockchain_name(&self) -> &'static str {
        "Polkadot"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polkadot_address_validation() {
        let handler = PolkadotHandler::new();

        // Generate a real Polkadot address to test validation
        let private_key = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let result = handler.derive_from_private_key(private_key);
        assert!(result.is_ok());
        let wallet_keys = result.unwrap();

        // Test with the generated address
        assert!(handler.validate_address(&wallet_keys.address));

        // Test with known invalid addresses
        assert!(!handler.validate_address("too_short"));
        assert!(!handler.validate_address("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2")); // Bitcoin address
        assert!(!handler.validate_address("0x742d35Cc6634C0532925a3b8D432C3475CB978B3")); // Ethereum address
        assert!(!handler.validate_address("")); // Empty string
        assert!(!handler.validate_address("polkadot")); // Invalid format
        assert!(!handler.validate_address("1ABCD@#$invalid_chars_here_should_fail")); // Invalid chars
    }

    #[test]
    fn test_polkadot_private_key_derivation() {
        let handler = PolkadotHandler::new();
        let private_key = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

        let result = handler.derive_from_private_key(private_key);
        assert!(result.is_ok());

        let keys = result.unwrap();
        assert_eq!(keys.private_key, private_key);
        assert!(keys.public_key.len() == 64); // 32 bytes in hex
        assert!(handler.validate_address(&keys.address));

        // Print the generated address for debugging
        println!("Generated Polkadot address: {}", keys.address);

        // Verify Polkadot address format (SS58-encoded, starts with '1' for mainnet)
        assert!(keys.address.starts_with('1'));
        assert!(keys.address.len() >= 47 && keys.address.len() <= 48);
        assert!(keys.address.chars().all(|c| c.is_ascii_alphanumeric()));
    }
}