use anyhow::{Result, Context};
use crate::blockchain::{BlockchainHandler, WalletKeys, SupportedBlockchain};
use crate::crypto::ed25519_utils::private_key_to_public_key_ed25519;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::signer::SeedDerivable;
use std::str::FromStr;
use bip39::Mnemonic;
use slip10_ed25519::derive_ed25519_private_key;

pub struct SolanaHandler;

impl SolanaHandler {
    pub fn new() -> Self {
        Self
    }
}

impl SolanaHandler {
    /// Derive Solana keypair using proper SLIP-0010 ed25519 derivation
    fn derive_solana_keypair_from_mnemonic(
        &self,
        mnemonic: &str,
        passphrase: Option<&str>,
        derivation_path: &str,
    ) -> Result<(Keypair, String)> {
        // 1) Parse BIP39 mnemonic and generate 64-byte seed
        let mnemonic_obj = Mnemonic::from_str(mnemonic)
            .context("Invalid BIP-39 mnemonic")?;
        let seed = mnemonic_obj.to_seed(passphrase.unwrap_or(""));

        // 2) Parse derivation path (e.g., "m/44'/501'/0'/0'") to indices
        let path_indices = self.parse_derivation_path_to_indices(derivation_path)?;

        // 3) Use slip10_ed25519 library for SLIP-0010 derivation
        let ed25519_seed = derive_ed25519_private_key(&seed, &path_indices);

        // 4) Create Solana Keypair from 32-byte ed25519 seed
        let keypair = Keypair::from_seed(&ed25519_seed)
            .map_err(|e| anyhow::anyhow!("Failed to create Keypair from seed: {}", e))?;
        let address = keypair.pubkey().to_string();

        Ok((keypair, address))
    }

    /// Parse BIP44 derivation path like "m/44'/501'/0'/0'" into simple indices
    fn parse_derivation_path_to_indices(&self, path: &str) -> Result<Vec<u32>> {
        if !path.starts_with("m/") {
            anyhow::bail!("Derivation path must start with 'm/'");
        }

        let path_str = &path[2..]; // Remove "m/"
        if path_str.is_empty() {
            return Ok(vec![]);
        }

        let mut indices = Vec::new();
        for component in path_str.split('/') {
            let index_str = if component.ends_with('\'') {
                &component[..component.len() - 1]
            } else {
                component
            };

            let index: u32 = index_str.parse()
                .context("Invalid derivation path index")?;

            indices.push(index);
        }

        Ok(indices)
    }
}

impl BlockchainHandler for SolanaHandler {
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
            None => SupportedBlockchain::Solana.get_default_derivation_path(account, address_index),
        };

        // Use proper SLIP-0010 ed25519 derivation
        let (keypair, address) = self.derive_solana_keypair_from_mnemonic(
            mnemonic,
            passphrase,
            &derivation_path,
        )?;

        let keypair_bytes = keypair.to_bytes();
        let private_key = &keypair_bytes[0..32];  // First 32 bytes are private key
        let public_key = keypair.pubkey().to_bytes();

        Ok(WalletKeys::new_simple(
            hex::encode(private_key),
            hex::encode(&public_key),
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
            return Err(anyhow::anyhow!("Solana private key must be 32 bytes"));
        }
        
        // Derive public key
        let public_key_bytes = private_key_to_public_key_ed25519(&private_key_bytes)?;
        
        // Create Solana Pubkey from public key bytes
        let pubkey_bytes: [u8; 32] = public_key_bytes.clone().try_into()
            .map_err(|_| anyhow::anyhow!("Invalid public key length for Solana"))?;
        let pubkey = Pubkey::from(pubkey_bytes);
        let address = pubkey.to_string();

        Ok(WalletKeys::new_simple(
            hex::encode(&private_key_bytes),
            hex::encode(&public_key_bytes),
            address,
            "imported".to_string(),
        ))
    }
    
    fn validate_address(&self, address: &str) -> bool {
        // Use official Solana SDK for address validation
        Pubkey::from_str(address).is_ok()
    }
    
    fn get_blockchain_name(&self) -> &'static str {
        "solana"
    }
}

impl SolanaHandler {
    /// Create Solana Keypair from private key bytes (32-byte seed)
    pub fn private_key_to_keypair(&self, private_key_bytes: &[u8]) -> Result<Keypair> {
        if private_key_bytes.len() != 32 {
            anyhow::bail!("Invalid private key length for Solana keypair: expected 32 bytes");
        }

        // Convert to 32-byte array and create keypair from seed
        let seed: [u8; 32] = private_key_bytes.try_into().unwrap();
        Keypair::from_seed(&seed)
            .map_err(|e| anyhow::anyhow!("Failed to create Keypair from seed: {}", e))
    }
    
    /// Generate Solana keypair in the format expected by Solana CLI tools
    pub fn to_solana_cli_format(&self, private_key_bytes: &[u8]) -> Result<String> {
        let keypair = self.private_key_to_keypair(private_key_bytes)?;
        Ok(serde_json::to_string(&keypair.to_bytes().to_vec())?)
    }
    
    /// Parse Solana CLI keypair format back to private key
    pub fn from_solana_cli_format(&self, keypair_json: &str) -> Result<Vec<u8>> {
        let keypair_array: Vec<u8> = serde_json::from_str(keypair_json)
            .context("Invalid Solana CLI keypair format")?;
        
        if keypair_array.len() != 64 {
            return Err(anyhow::anyhow!("Invalid Solana keypair length"));
        }
        
        // Return just the private key portion (first 32 bytes)
        Ok(keypair_array[0..32].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solana_from_mnemonic() {
        let handler = SolanaHandler::new();
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        
        let result = handler.derive_from_mnemonic(mnemonic, None, 0, 0, None);
        assert!(result.is_ok());
        
        let keys = result.unwrap();
        assert!(!keys.private_key.is_empty());
        assert!(!keys.public_key.is_empty());
        assert!(!keys.address.is_empty());
        
        // Solana addresses should be valid base58
        assert!(handler.validate_address(&keys.address));
    }
    
    #[test]
    fn test_solana_from_private_key() {
        let handler = SolanaHandler::new();
        let private_key = "0000000000000000000000000000000000000000000000000000000000000001";
        
        let result = handler.derive_from_private_key(private_key);
        assert!(result.is_ok());
        
        let keys = result.unwrap();
        assert_eq!(keys.private_key, private_key);
        assert!(!keys.public_key.is_empty());
        assert!(!keys.address.is_empty());
        assert!(handler.validate_address(&keys.address));
    }
    
    #[test]
    fn test_address_validation() {
        let handler = SolanaHandler::new();
        
        // Valid Solana address (example)
        let valid_address = "11111111111111111111111111111111";
        assert!(handler.validate_address(valid_address));
        
        // Invalid addresses
        assert!(!handler.validate_address("invalid_address"));
        assert!(!handler.validate_address("too_short"));
        assert!(!handler.validate_address("0x742d35Cc6634C0532925a3b8D322C8e1c6a331cb")); // ETH address
    }
    
    #[test]
    fn test_keypair_conversion() {
        let handler = SolanaHandler::new();
        let private_key = hex::decode("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        
        // Test Solana keypair generation
        let keypair = handler.private_key_to_keypair(&private_key);
        assert!(keypair.is_ok());

        let kp = keypair.unwrap();
        let kp_bytes = kp.to_bytes();
        assert_eq!(kp_bytes.len(), 64);
        assert_eq!(&kp_bytes[0..32], &private_key);
        
        // Test CLI format conversion
        let cli_format = handler.to_solana_cli_format(&private_key);
        assert!(cli_format.is_ok());
        
        let cli_json = cli_format.unwrap();
        let parsed_private = handler.from_solana_cli_format(&cli_json);
        assert!(parsed_private.is_ok());
        assert_eq!(parsed_private.unwrap(), private_key);
    }
    
    #[test]
    fn test_derivation_path() {
        let handler = SolanaHandler::new();
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

        // Test custom path
        let custom_path = "m/44'/501'/1'/0'";
        let result = handler.derive_from_mnemonic(mnemonic, None, 0, 0, Some(custom_path));
        assert!(result.is_ok());

        let keys = result.unwrap();
        assert_eq!(keys.derivation_path, custom_path);
    }

    #[test]
    fn test_solana_derivation_paths() {
        let handler = SolanaHandler::new();
        // Standard test mnemonic (BIP39 test vector)
        let test_mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

        // Known addresses for this test mnemonic with Solana paths (generated with this implementation)
        let expected_3_level = "GjJyeC1r2RgkuoCWMyPYkCWSGSGLcz266EaAkLA27AhL";  // m/44'/501'/0'
        let expected_4_level = "HAgk14JpMQLgt6rVgv7cBQFJWFto5Dqxi472uT3DKpqk";  // m/44'/501'/0'/0'

        // Test 3-level path (Trust Wallet style)
        println!("Testing 3-level path: m/44'/501'/0'");
        let result_3_level = handler.derive_from_mnemonic(test_mnemonic, None, 0, 0, Some("m/44'/501'/0'"));
        assert!(result_3_level.is_ok(), "Failed to derive keys for 3-level path");
        let keys_3_level = result_3_level.unwrap();
        println!("  Generated 3-level address: {}", keys_3_level.address);
        assert_eq!(keys_3_level.address, expected_3_level, "3-level path address mismatch");

        // Test 4-level path (BIP-44 standard)
        println!("Testing 4-level path: m/44'/501'/0'/0'");
        let result_4_level = handler.derive_from_mnemonic(test_mnemonic, None, 0, 0, Some("m/44'/501'/0'/0'"));
        assert!(result_4_level.is_ok(), "Failed to derive keys for 4-level path");
        let keys_4_level = result_4_level.unwrap();
        println!("  Generated 4-level address: {}", keys_4_level.address);
        assert_eq!(keys_4_level.address, expected_4_level, "4-level path address mismatch");

        // Ensure they're different (important for security)
        assert_ne!(keys_3_level.address, keys_4_level.address, "3-level and 4-level addresses should be different");

        println!("✅ All Solana derivation paths working correctly");
    }

    #[test]
    fn test_parse_derivation_path() {
        let handler = SolanaHandler::new();

        // Test standard path
        let path = "m/44'/501'/0'/0'";
        let result = handler.parse_derivation_path_to_indices(path);
        assert!(result.is_ok());
        let indices = result.unwrap();
        assert_eq!(indices, vec![44, 501, 0, 0]);

        // Test 3-level path
        let path = "m/44'/501'/0'";
        let result = handler.parse_derivation_path_to_indices(path);
        assert!(result.is_ok());
        let indices = result.unwrap();
        assert_eq!(indices, vec![44, 501, 0]);

        // Test invalid path
        let path = "invalid";
        let result = handler.parse_derivation_path_to_indices(path);
        assert!(result.is_err());
    }
}