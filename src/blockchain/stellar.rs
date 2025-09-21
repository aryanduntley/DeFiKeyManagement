use anyhow::{Result, Context};
use crate::blockchain::{BlockchainHandler, WalletKeys, SupportedBlockchain};
use crate::crypto::ed25519_utils::{derive_slip10_ed25519_key_from_mnemonic, private_key_to_public_key_ed25519};
use stellar_base::crypto::{PublicKey as StellarPublicKey, SodiumKeyPair};

pub struct StellarHandler;

impl StellarHandler {
    pub fn new() -> Self {
        Self
    }
}

impl BlockchainHandler for StellarHandler {
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
            None => SupportedBlockchain::Stellar.get_default_derivation_path(account, address_index),
        };
        
        // Derive ed25519 keys using SLIP-0010 (matches iancoleman.io/bip39)
        let (private_key_bytes, public_key_bytes) = derive_slip10_ed25519_key_from_mnemonic(
            mnemonic,
            passphrase,
            &derivation_path,
        )?;
        
        // Generate Stellar address from public key
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
            return Err(anyhow::anyhow!("Stellar private key must be 32 bytes"));
        }
        
        // Derive public key
        let public_key_bytes = private_key_to_public_key_ed25519(&private_key_bytes)?;
        
        // Generate Stellar address
        let address = self.public_key_to_address(&public_key_bytes)?;
        
        Ok(WalletKeys::new_simple(
            hex::encode(&private_key_bytes),
            hex::encode(&public_key_bytes),
            address,
            "imported".to_string(),
        ))
    }
    
    fn validate_address(&self, address: &str) -> bool {
        // Use stellar-base library to validate address
        StellarPublicKey::from_account_id(address).is_ok()
    }
    
    fn get_blockchain_name(&self) -> &'static str {
        "stellar"
    }
}

impl StellarHandler {
    /// Convert hex private key to Stellar secret key format (starting with 'S')
    pub fn hex_private_key_to_stellar_secret(&self, hex_private_key: &str) -> Result<String> {
        // Parse hex private key
        let private_key_bytes = hex::decode(hex_private_key)
            .context("Invalid hexadecimal private key")?;

        if private_key_bytes.len() != 32 {
            return Err(anyhow::anyhow!("Stellar private key must be 32 bytes"));
        }

        // Create Stellar SodiumKeyPair from raw ed25519 seed
        let keypair = SodiumKeyPair::from_seed_bytes(&private_key_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to create Stellar keypair: {}", e))?;

        // Get the secret key in Stellar format (starts with 'S')
        Ok(keypair.secret_key().secret_seed())
    }

    fn public_key_to_address(&self, public_key_bytes: &[u8]) -> Result<String> {
        if public_key_bytes.len() != 32 {
            return Err(anyhow::anyhow!("Invalid public key length for Stellar"));
        }
        
        // Convert bytes to stellar PublicKey
        let stellar_public_key = StellarPublicKey::from_slice(public_key_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to create Stellar public key: {}", e))?;
        
        // Get account ID from public key
        Ok(stellar_public_key.account_id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stellar_from_mnemonic() {
        let handler = StellarHandler::new();
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        
        let result = handler.derive_from_mnemonic(mnemonic, None, 0, 0, None);
        assert!(result.is_ok());
        
        let keys = result.unwrap();
        assert!(!keys.private_key.is_empty());
        assert!(!keys.public_key.is_empty());
        assert!(keys.address.starts_with('G'));
        assert_eq!(keys.address.len(), 56);
        assert!(handler.validate_address(&keys.address));
    }
    
    #[test]
    fn test_stellar_from_private_key() {
        let handler = StellarHandler::new();
        let private_key = "0000000000000000000000000000000000000000000000000000000000000001";
        
        let result = handler.derive_from_private_key(private_key);
        assert!(result.is_ok());
        
        let keys = result.unwrap();
        assert_eq!(keys.private_key, private_key);
        assert!(!keys.public_key.is_empty());
        assert!(keys.address.starts_with('G'));
        assert!(handler.validate_address(&keys.address));
    }
    
    #[test]
    fn test_address_validation() {
        let handler = StellarHandler::new();
        
        // Invalid addresses
        assert!(!handler.validate_address("invalid_address"));
        assert!(!handler.validate_address("0x742d35Cc6634C0532925a3b8D322C8e1c6a331cb")); // ETH address
        assert!(!handler.validate_address("GAAAAA")); // Too short
        
        // Test with a generated address
        let private_key = "1111111111111111111111111111111111111111111111111111111111111111";
        let result = handler.derive_from_private_key(private_key);
        assert!(result.is_ok());
        
        let keys = result.unwrap();
        assert!(handler.validate_address(&keys.address));
    }
    
    #[test]
    fn test_stellar_base_integration() {
        let handler = StellarHandler::new();
        let private_key = "2222222222222222222222222222222222222222222222222222222222222222";

        let keys = handler.derive_from_private_key(private_key).unwrap();

        // Test that the address is valid using stellar-base validation
        assert!(handler.validate_address(&keys.address));

        // Test that the address has the correct Stellar format
        assert!(keys.address.starts_with('G'));
        assert_eq!(keys.address.len(), 56);
    }

    #[test]
    fn test_stellar_expected_addresses() {
        let handler = StellarHandler::new();
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

        // Test expected addresses from iancoleman.io/bip39
        let expected = [
            ("m/44'/148'/0'", "GB3JDWCQJCWMJ3IILWIGDTQJJC5567PGVEVXSCVPEQOTDN64VJBDQBYX", "SBUV3MRWKNS6AYKZ6E6MOUVF2OYMON3MIUASWL3JLY5E3ISDJFELYBRZ"),
            ("m/44'/148'/1'", "GDVSYYTUAJ3ACHTPQNSTQBDQ4LDHQCMNY4FCEQH5TJUMSSLWQSTG42MV", "SCHDCVCWGAKGIMTORV6K5DYYV3BY4WG3RA4M6MCBGJLHUCWU2MC6DL66"),
            ("m/44'/148'/2'", "GBFPWBTN4AXHPWPTQVQBP4KRZ2YVYYOGRMV2PEYL2OBPPJDP7LECEVHR", "SAPLVTLUXSDLFRDGCCFLPDZMTCEVMP3ZXTM74EBJCVKZKM34LGQPF7K3"),
            ("m/44'/148'/3'", "GCCCOWAKYVFY5M6SYHOW33TSNC7Z5IBRUEU2XQVVT34CIZU7CXZ4OQ4O", "SDQYXOP2EAUZP4YOEQ5BUJIQ3RDSP5XV4ZFI6C5Y3QCD5Y63LWPXT7PW"),
        ];

        for (path, expected_address, expected_private_key) in expected {
            let account_index = path.split('/').last().unwrap().trim_end_matches('\'').parse::<u32>().unwrap();
            let result = handler.derive_from_mnemonic(mnemonic, None, account_index, 0, Some(path));

            assert!(result.is_ok(), "Failed to derive keys for path {}", path);
            let keys = result.unwrap();

            println!("Path: {}", path);
            println!("Expected Address: {}", expected_address);
            println!("Actual Address:   {}", keys.address);
            println!("Expected Private: {}", expected_private_key);
            println!("Actual Private:   {}", keys.private_key);
            println!("Actual Public:    {}", keys.public_key);
            println!("---");

            // Verify the address matches expected output from iancoleman.io/bip39
            assert_eq!(keys.address, expected_address, "Address mismatch for path {}", path);

            // Verify standard Stellar address format
            assert!(keys.address.starts_with('G'));
            assert_eq!(keys.address.len(), 56);
        }
    }
}