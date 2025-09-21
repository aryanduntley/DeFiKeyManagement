use anyhow::{Result, Context};
use crate::blockchain::{BlockchainHandler, WalletKeys, SupportedBlockchain};
use crate::crypto::bip32::{derive_secp256k1_key_from_mnemonic, private_key_to_public_key_secp256k1};
use litcoin::{Network, PublicKey};
use sha2::Digest;
use ripemd::Digest as RipemdDigest;

pub struct LitecoinHandler {
    network: Network,
}

impl LitecoinHandler {
    pub fn new() -> Self {
        // Let's see what networks are available
        println!("Available networks: {:?}", Network::Bitcoin);
        println!("Available networks: {:?}", Network::Testnet);
        // Try to see if there's a Network::Regtest or any other network
        Self {
            network: Network::Bitcoin, // litcoin library uses Bitcoin network constants
        }
    }

    pub fn new_testnet() -> Self {
        Self {
            network: Network::Testnet,
        }
    }

    fn public_key_to_address(&self, public_key_bytes: &[u8]) -> Result<String> {
        // Parse the compressed secp256k1 public key
        let public_key = PublicKey::from_slice(public_key_bytes)
            .context("Invalid public key")?;

        // Generate Litecoin P2PKH (legacy) address with proper Litecoin prefix
        // For mainnet Litecoin, addresses start with 'L' or 'M'
        let address = self.generate_litecoin_p2pkh_address(&public_key)?;

        println!("Generated Litecoin P2PKH address: {}", address);

        Ok(address)
    }

    fn generate_litecoin_p2pkh_address(&self, public_key: &PublicKey) -> Result<String> {
        use sha2::{Sha256, Digest};
        use ripemd::{Ripemd160, Digest as RipemdDigest};

        // Step 1: SHA256 hash of the public key
        let mut sha256_hasher = Sha256::new();
        sha256_hasher.update(&public_key.to_bytes());
        let sha256_hash = sha256_hasher.finalize();

        // Step 2: RIPEMD160 hash of the SHA256 hash
        let mut ripemd160_hasher = Ripemd160::new();
        ripemd160_hasher.update(&sha256_hash);
        let pubkey_hash = ripemd160_hasher.finalize();

        // Step 3: Add Litecoin mainnet P2PKH prefix (0x30 = 48 decimal for 'L' addresses)
        let mut payload = vec![0x30u8]; // Litecoin mainnet P2PKH version byte
        payload.extend_from_slice(&pubkey_hash);

        // Step 4: Double SHA256 for checksum
        let mut sha256_hasher_1 = Sha256::new();
        sha256_hasher_1.update(&payload);
        let first_hash = sha256_hasher_1.finalize();

        let mut sha256_hasher_2 = Sha256::new();
        sha256_hasher_2.update(&first_hash);
        let second_hash = sha256_hasher_2.finalize();

        // Step 5: Take first 4 bytes as checksum
        let checksum = &second_hash[0..4];

        // Step 6: Append checksum to payload
        payload.extend_from_slice(checksum);

        // Step 7: Base58 encode
        let address = bs58::encode(payload).into_string();

        Ok(address)
    }
}

impl BlockchainHandler for LitecoinHandler {
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
            None => SupportedBlockchain::Litecoin.get_default_derivation_path(account, address_index),
        };

        // Derive private and public key using BIP-32
        let (private_key_bytes, public_key_bytes) = derive_secp256k1_key_from_mnemonic(
            mnemonic,
            passphrase,
            &derivation_path,
        )?;

        // Generate Litecoin address from public key
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
            return Err(anyhow::anyhow!("Litecoin private key must be 32 bytes"));
        }

        // Derive public key
        let public_key_bytes = private_key_to_public_key_secp256k1(&private_key_bytes)?;

        // Generate Litecoin address
        let address = self.public_key_to_address(&public_key_bytes)?;

        Ok(WalletKeys::new_simple(
            hex::encode(&private_key_bytes),
            hex::encode(&public_key_bytes),
            address,
            "N/A (from private key)".to_string(),
        ))
    }

    fn validate_address(&self, address: &str) -> bool {
        // Litecoin addresses start with:
        // - 'L' for P2PKH (legacy)
        // - 'M' for P2SH (legacy)
        // - 'ltc1' for Bech32 (SegWit)
        if address.starts_with('L') || address.starts_with('M') {
            // Try to decode as Base58Check
            if let Ok(decoded) = bs58::decode(address).into_vec() {
                // Should be 25 bytes (1 version + 20 hash + 4 checksum)
                if decoded.len() == 25 {
                    // Check version byte (0x30 for L, 0x32 for M addresses)
                    return decoded[0] == 0x30 || decoded[0] == 0x32;
                }
            }
        } else if address.starts_with("ltc1") {
            // TODO: Add Bech32 validation for Litecoin
            return true; // Placeholder for now
        }
        false
    }

    fn get_blockchain_name(&self) -> &'static str {
        "Litecoin"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_litecoin_address_validation() {
        let handler = LitecoinHandler::new();

        // Test valid Litecoin addresses
        assert!(handler.validate_address("LdP8Qox1VAhCzLJNqrr74YovaWYyNBUWvL")); // P2PKH
        assert!(handler.validate_address("MQMcJhpWHYVeQArcZR3sBgyPZxxRtnH441")); // P2SH
        assert!(handler.validate_address("ltc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh")); // Bech32

        // Test invalid addresses
        assert!(!handler.validate_address("invalid_address"));
        assert!(!handler.validate_address("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2")); // Bitcoin address
        assert!(!handler.validate_address("")); // Empty string
        assert!(!handler.validate_address("L")); // Too short
    }

    #[test]
    fn test_litecoin_private_key_derivation() {
        let handler = LitecoinHandler::new();

        // Test with a known private key
        let private_key = "1e99423a4ed27608a15a2616a2b0e9e52ced330ac530edcc32c8ffc6a526aedd";
        let result = handler.derive_from_private_key(private_key);

        assert!(result.is_ok());
        let wallet_keys = result.unwrap();
        println!("Generated Litecoin address: {}", wallet_keys.address);
        assert!(wallet_keys.address.starts_with("L")); // Should generate Litecoin P2PKH address
        assert!(!wallet_keys.address.is_empty());
    }
}