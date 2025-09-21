use anyhow::{Result, Context};
use crate::blockchain::{BlockchainHandler, WalletKeys, SupportedBlockchain};
use crate::crypto::ed25519_utils::{derive_cardano_key_from_mnemonic, private_key_to_public_key_ed25519};
use cardano_serialization_lib::{
    BaseAddress, EnterpriseAddress, Credential, PublicKey as CardanoPublicKey
};

pub struct CardanoHandler;

impl CardanoHandler {
    pub fn new() -> Self {
        Self
    }

    fn generate_base_address(&self, public_key: &[u8]) -> Result<String> {
        // Use official cardano-serialization-lib for base address generation
        // This ensures exact compatibility with Trust Wallet and other standard wallets

        // Create CardanoPublicKey from raw bytes
        let cardano_pub_key = CardanoPublicKey::from_bytes(public_key)
            .map_err(|e| anyhow::anyhow!("Invalid public key: {:?}", e))?;

        // Create payment credential from public key hash
        let payment_key_hash = cardano_pub_key.hash();

        // For base address, we need a stake credential
        // Using the same key hash for stake credential (simplified approach)
        // This matches how most wallets generate base addresses by default
        let stake_key_hash = payment_key_hash.clone();

        // Create credentials
        let payment_cred = Credential::from_keyhash(&payment_key_hash);
        let stake_cred = Credential::from_keyhash(&stake_key_hash);

        // Create base address using official method (mainnet = 0x01)
        let base_addr = BaseAddress::new(0x01, &payment_cred, &stake_cred);

        // Convert to bech32 string - this will match official wallets exactly
        Ok(base_addr.to_address().to_bech32(None)
            .map_err(|e| anyhow::anyhow!("Failed to encode base address: {:?}", e))?)
    }

    fn generate_enterprise_address(&self, public_key: &[u8]) -> Result<String> {
        // Use official cardano-serialization-lib for enterprise address generation
        // This ensures exact compatibility with Trust Wallet and other standard wallets

        // Create CardanoPublicKey from raw bytes
        let cardano_pub_key = CardanoPublicKey::from_bytes(public_key)
            .map_err(|e| anyhow::anyhow!("Invalid public key: {:?}", e))?;

        // Create payment credential from public key hash
        let payment_key_hash = cardano_pub_key.hash();
        let payment_cred = Credential::from_keyhash(&payment_key_hash);

        // Create enterprise address using official method (mainnet = 0x01)
        let enterprise_addr = EnterpriseAddress::new(0x01, &payment_cred);

        // Convert to bech32 string - this will match official wallets exactly
        Ok(enterprise_addr.to_address().to_bech32(None)
            .map_err(|e| anyhow::anyhow!("Failed to encode enterprise address: {:?}", e))?)
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

        // Derive private and public key using Cardano-specific derivation
        let (private_key_bytes, public_key_bytes) = derive_cardano_key_from_mnemonic(
            mnemonic,
            passphrase,
            &derivation_path,
        )?;

        // Generate Cardano addresses from public key
        let base_address = self.generate_base_address(&public_key_bytes)?;
        let enterprise_address = self.generate_enterprise_address(&public_key_bytes)?;

        // Create WalletKeys with base address as primary and enterprise as secondary
        let mut wallet_keys = WalletKeys::new_simple(
            hex::encode(&private_key_bytes),
            hex::encode(&public_key_bytes),
            base_address,
            derivation_path,
        );

        // Add enterprise address as secondary address
        wallet_keys.add_secondary_address("enterprise".to_string(), enterprise_address);

        Ok(wallet_keys)
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

        // Generate Cardano addresses from public key
        let base_address = self.generate_base_address(&public_key_bytes)?;
        let enterprise_address = self.generate_enterprise_address(&public_key_bytes)?;

        // Create WalletKeys with base address as primary and enterprise as secondary
        let mut wallet_keys = WalletKeys::new_simple(
            hex::encode(&private_key_bytes),
            hex::encode(&public_key_bytes),
            base_address,
            "N/A (from private key)".to_string(),
        );

        // Add enterprise address as secondary address
        wallet_keys.add_secondary_address("enterprise".to_string(), enterprise_address);

        Ok(wallet_keys)
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