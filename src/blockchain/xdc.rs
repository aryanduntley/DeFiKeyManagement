use anyhow::{Result, Context};
use crate::blockchain::{BlockchainHandler, WalletKeys};
use alloy_primitives::Address;
use k256::elliptic_curve::sec1::ToEncodedPoint;

pub struct XdcHandler;

impl XdcHandler {
    pub fn new() -> Self {
        Self
    }

    fn public_key_to_address(&self, public_key_bytes: &[u8]) -> Result<String> {
        if public_key_bytes.len() != 33 {
            anyhow::bail!("XDC public key must be 33 bytes (compressed), got {}", public_key_bytes.len());
        }

        // Convert compressed public key to uncompressed format
        let public_key = k256::PublicKey::from_sec1_bytes(public_key_bytes)
            .context("Invalid secp256k1 public key")?;
        let uncompressed_bytes = public_key.to_encoded_point(false);
        let uncompressed_key = uncompressed_bytes.as_bytes();

        // Skip the 0x04 prefix, use the remaining 64 bytes
        let key_bytes = &uncompressed_key[1..];

        // Generate Ethereum-style address using Keccak256
        use sha3::{Digest, Keccak256};
        let mut hasher = Keccak256::new();
        hasher.update(key_bytes);
        let hash = hasher.finalize();

        // Take last 20 bytes and create XDC address with 'xdc' prefix
        let address_bytes = &hash[12..];
        let eth_address = Address::from_slice(address_bytes);

        // Convert to XDC format: replace '0x' with 'xdc'
        let eth_address_str = format!("{:?}", eth_address);
        let xdc_address = eth_address_str.replacen("0x", "xdc", 1);

        Ok(xdc_address)
    }

    fn validate_xdc_address(&self, address: &str) -> bool {
        // XDC addresses should start with 'xdc' and be 43 characters total (xdc + 40 hex chars)
        if !address.starts_with("xdc") || address.len() != 43 {
            return false;
        }

        // Convert to Ethereum format for validation
        let eth_format = address.replacen("xdc", "0x", 1);

        // Try to parse as Ethereum address to validate format
        eth_format.parse::<Address>().is_ok()
    }

    fn secp256k1_public_key_from_private(&self, private_key_bytes: &[u8]) -> Result<Vec<u8>> {
        use k256::ecdsa::SigningKey;

        let signing_key = SigningKey::from_bytes(private_key_bytes.into())
            .context("Invalid secp256k1 private key")?;

        let verifying_key = signing_key.verifying_key();
        let public_key_point = verifying_key.to_encoded_point(true); // compressed
        Ok(public_key_point.as_bytes().to_vec())
    }
}

impl BlockchainHandler for XdcHandler {
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
                // XDC uses SLIP-0044 coin type 550: m/44'/550'/0'/0/0
                format!("m/44'/550'/{}'/{}/{}", account, 0, address_index)
            },
        };

        // Use secp256k1 key derivation for XDC (not ed25519)
        use crate::crypto::bip32::{derive_secp256k1_key_from_mnemonic};
        let (private_key_bytes, _) = derive_secp256k1_key_from_mnemonic(
            mnemonic,
            passphrase,
            &derivation_path,
        )?;

        // Derive compressed public key from private key
        let public_key_bytes = self.secp256k1_public_key_from_private(&private_key_bytes)?;

        // Generate XDC address from public key
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
            anyhow::bail!("XDC private key must be 32 bytes");
        }

        // Derive compressed public key from private key
        let public_key_bytes = self.secp256k1_public_key_from_private(&private_key_bytes)?;

        // Generate XDC address from public key
        let address = self.public_key_to_address(&public_key_bytes)?;

        Ok(WalletKeys::new_simple(
            hex::encode(&private_key_bytes),
            hex::encode(&public_key_bytes),
            address,
            "Imported from private key".to_string(),
        ))
    }

    fn validate_address(&self, address: &str) -> bool {
        self.validate_xdc_address(address)
    }

    fn get_blockchain_name(&self) -> &'static str {
        "XDC"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xdc_address_validation() {
        let handler = XdcHandler::new();

        // Generate a real XDC address to test validation
        let private_key = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let result = handler.derive_from_private_key(private_key);
        assert!(result.is_ok());
        let wallet_keys = result.unwrap();

        // Test with the generated address
        assert!(handler.validate_address(&wallet_keys.address));

        // Test with known invalid addresses
        assert!(!handler.validate_address("invalid_address"));
        assert!(!handler.validate_address("0x742d35Cc6634C0532925a3b8D432C3475CB978B3")); // Ethereum format
        assert!(!handler.validate_address("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2")); // Bitcoin address
        assert!(!handler.validate_address("")); // Empty string
        assert!(!handler.validate_address("xdc")); // Too short

        // Test proper XDC format
        assert!(wallet_keys.address.starts_with("xdc"));
        assert_eq!(wallet_keys.address.len(), 43);
    }

    #[test]
    fn test_xdc_private_key_derivation() {
        let handler = XdcHandler::new();
        let private_key = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

        let result = handler.derive_from_private_key(private_key);
        assert!(result.is_ok());

        let keys = result.unwrap();

        // Print the generated address for debugging
        println!("Generated XDC address: {}", keys.address);
        println!("Address length: {}", keys.address.len());
        println!("Validation result: {}", handler.validate_address(&keys.address));

        assert_eq!(keys.private_key, private_key);
        assert!(keys.public_key.len() == 66); // 33 bytes in hex (compressed)
        assert!(handler.validate_address(&keys.address));

        // Verify XDC address format (xdc prefix and appropriate length)
        assert!(keys.address.starts_with("xdc"));
        assert_eq!(keys.address.len(), 43);
    }

    #[test]
    fn test_xdc_uses_correct_derivation_path() {
        let handler = XdcHandler::new();
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

        let result = handler.derive_from_mnemonic(mnemonic, None, 0, 0, None);
        assert!(result.is_ok());

        let keys = result.unwrap();
        assert_eq!(keys.derivation_path, "m/44'/550'/0'/0/0");
        assert!(keys.address.starts_with("xdc"));
        println!("XDC derivation path: {}", keys.derivation_path);
        println!("XDC address: {}", keys.address);
    }
}