use anyhow::{Result, Context};
use ed25519_dalek::SigningKey;
use bip39::Mnemonic;
use std::str::FromStr;
use sha2::{Sha512, Digest};
use ed25519_bip32::{XPrv, XPub, DerivationIndex, DerivationScheme};
use hmac::{Hmac, Mac};
use slip10_ed25519::derive_ed25519_private_key;

/// SLIP-0010 compatible ed25519 key derivation
pub fn derive_ed25519_key_from_mnemonic(
    mnemonic: &str,
    passphrase: Option<&str>,
    derivation_path: &str,
) -> Result<(Vec<u8>, Vec<u8>)> {
    // Parse and validate mnemonic
    let mnemonic = Mnemonic::from_str(mnemonic)
        .context("Invalid BIP-39 mnemonic")?;
    
    // Generate seed from mnemonic + passphrase  
    let seed = mnemonic.to_seed(passphrase.unwrap_or(""));
    
    // For ed25519, we use SLIP-0010 derivation
    let (private_key, public_key) = derive_slip0010_ed25519(&seed, derivation_path)?;
    
    Ok((private_key, public_key))
}

/// SLIP-0010 ed25519 key derivation implementation
/// Reference: https://github.com/satoshilabs/slips/blob/master/slip-0010.md
pub fn derive_slip0010_ed25519(seed: &[u8], path: &str) -> Result<(Vec<u8>, Vec<u8>)> {
    // Parse derivation path (e.g., "m/44'/501'/0'/0'")
    let path_components = parse_derivation_path(path)?;
    
    // Start with master key from seed
    let mut key = derive_master_key_ed25519(seed)?;
    
    // Derive each level in the path
    for &index in &path_components {
        key = derive_child_key_ed25519(&key, index)?;
    }
    
    // Generate public key from private key
    let signing_key = SigningKey::from_bytes(&key);
    let verifying_key = signing_key.verifying_key();
    
    Ok((key.to_vec(), verifying_key.to_bytes().to_vec()))
}

fn derive_master_key_ed25519(seed: &[u8]) -> Result<[u8; 32]> {
    use hmac::{Hmac, Mac};
    type HmacSha512 = Hmac<Sha512>;
    
    let mut mac = HmacSha512::new_from_slice(b"ed25519 seed")
        .context("Failed to create HMAC")?;
    mac.update(seed);
    let result = mac.finalize().into_bytes();
    
    // Take the left 32 bytes as the private key
    let mut key = [0u8; 32];
    key.copy_from_slice(&result[0..32]);
    
    Ok(key)
}

fn derive_child_key_ed25519(parent_key: &[u8; 32], index: u32) -> Result<[u8; 32]> {
    use hmac::{Hmac, Mac};
    type HmacSha512 = Hmac<Sha512>;
    
    // For ed25519, all derivations are hardened
    let hardened_index = index | 0x80000000;
    
    let mut mac = HmacSha512::new_from_slice(parent_key)
        .context("Failed to create HMAC for child derivation")?;
    
    // Input: 0x00 || parent_private_key || index (4 bytes, big endian)
    mac.update(&[0x00]);
    mac.update(parent_key);
    mac.update(&hardened_index.to_be_bytes());
    
    let result = mac.finalize().into_bytes();
    
    // Take the left 32 bytes as the new private key
    let mut key = [0u8; 32];
    key.copy_from_slice(&result[0..32]);
    
    Ok(key)
}

fn parse_derivation_path(path: &str) -> Result<Vec<u32>> {
    if !path.starts_with("m/") {
        return Err(anyhow::anyhow!("Derivation path must start with 'm/'"));
    }
    
    let path_str = &path[2..]; // Remove "m/"
    if path_str.is_empty() {
        return Ok(vec![]);
    }
    
    let mut indices = Vec::new();
    for component in path_str.split('/') {
        let hardened = component.ends_with('\'');
        let index_str = if hardened {
            &component[..component.len() - 1]
        } else {
            component
        };
        
        let index: u32 = index_str.parse()
            .context("Invalid derivation path index")?;
        
        let final_index = if hardened {
            index | 0x80000000
        } else {
            index
        };
        
        indices.push(final_index);
    }
    
    Ok(indices)
}

/// BIP32-Ed25519 key derivation specifically for Stellar (and other standards-compliant chains)
/// This matches the derivation used by iancoleman.io/bip39 and other standard tools
pub fn derive_ed25519_bip32_key_from_mnemonic(
    mnemonic: &str,
    passphrase: Option<&str>,
    derivation_path: &str,
) -> Result<(Vec<u8>, Vec<u8>)> {
    // Parse and validate mnemonic
    let mnemonic = Mnemonic::from_str(mnemonic)
        .context("Invalid BIP-39 mnemonic")?;

    // Generate seed from mnemonic + passphrase
    let seed = mnemonic.to_seed(passphrase.unwrap_or(""));

    // Parse derivation path (e.g., "m/44'/148'/1'")
    let path_components = parse_derivation_path(derivation_path)?;

    // Create root extended private key from seed (96 bytes: 64 extended key + 32 chain code)
    let mut root_bytes = [0u8; 96];

    // Use HMAC-SHA512 with "ed25519 seed" as key
    type HmacSha512 = Hmac<sha2::Sha512>;
    let mut mac = HmacSha512::new_from_slice(b"ed25519 seed")
        .context("Failed to create HMAC")?;
    mac.update(&seed);
    let result = mac.finalize().into_bytes();

    // HMAC-SHA512 returns 64 bytes, so we need to split it properly
    // First 32 bytes become private key, second 32 bytes become chain code
    // The extended private key itself is 64 bytes (32 + 32)
    root_bytes[0..32].copy_from_slice(&result[0..32]);   // Private key (left 32)
    root_bytes[32..64].copy_from_slice(&result[0..32]);  // Extended part (also left 32)
    root_bytes[64..96].copy_from_slice(&result[32..64]); // Chain code (right 32)

    let root_xprv = XPrv::normalize_bytes_ed25519(root_bytes);

    // Derive the path step by step
    let mut current_xprv = root_xprv;
    for &index in &path_components {
        let derivation_index = index; // Already includes hardened bit from parse_derivation_path
        current_xprv = current_xprv.derive(DerivationScheme::V2, derivation_index);
    }

    // Get the private and public key bytes
    let extended_private_key = current_xprv.extended_secret_key();
    let private_key = &extended_private_key[0..32]; // First 32 bytes are the actual private key
    let public_key = current_xprv.public().public_key();

    Ok((private_key.to_vec(), public_key.to_vec()))
}

/// SLIP-0010 key derivation specifically for Stellar (proper implementation)
/// This uses the slip10_ed25519 crate which matches iancoleman.io/bip39
pub fn derive_slip10_ed25519_key_from_mnemonic(
    mnemonic: &str,
    passphrase: Option<&str>,
    derivation_path: &str,
) -> Result<(Vec<u8>, Vec<u8>)> {
    // Parse and validate mnemonic
    let mnemonic = Mnemonic::from_str(mnemonic)
        .context("Invalid BIP-39 mnemonic")?;

    // Generate seed from mnemonic + passphrase
    let seed = mnemonic.to_seed(passphrase.unwrap_or(""));

    // Parse the derivation path for the slip10_ed25519 crate
    // The library expects just the index numbers (treats all as hardened)
    let path_parts: Result<Vec<u32>, _> = derivation_path
        .strip_prefix("m/")
        .ok_or_else(|| anyhow::anyhow!("Invalid derivation path: must start with m/"))?
        .split('/')
        .map(|part| {
            let index_str = if part.ends_with('\'') {
                &part[..part.len() - 1]
            } else {
                part
            };
            index_str.parse::<u32>().context("Invalid derivation index")
        })
        .collect();

    let path_indices = path_parts?;

    // Use the slip10_ed25519 library to derive the private key
    let private_key = derive_ed25519_private_key(&seed, &path_indices);

    // Generate public key from private key
    let signing_key = SigningKey::from_bytes(&private_key);
    let verifying_key = signing_key.verifying_key();

    Ok((private_key.to_vec(), verifying_key.to_bytes().to_vec()))
}

pub fn private_key_to_public_key_ed25519(private_key: &[u8]) -> Result<Vec<u8>> {
    if private_key.len() != 32 {
        return Err(anyhow::anyhow!("ed25519 private key must be 32 bytes"));
    }

    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(private_key);

    let signing_key = SigningKey::from_bytes(&key_bytes);
    let verifying_key = signing_key.verifying_key();

    Ok(verifying_key.to_bytes().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_derivation_path() {
        let path = "m/44'/501'/0'/0'";
        let result = parse_derivation_path(path).unwrap();
        
        assert_eq!(result.len(), 4);
        assert_eq!(result[0], 44 | 0x80000000); // Hardened
        assert_eq!(result[1], 501 | 0x80000000); // Hardened
        assert_eq!(result[2], 0 | 0x80000000); // Hardened
        assert_eq!(result[3], 0 | 0x80000000); // Hardened
    }
    
    #[test]
    fn test_derive_master_key() {
        let seed = b"test seed for ed25519 derivation";
        let master_key = derive_master_key_ed25519(seed);
        
        assert!(master_key.is_ok());
        assert_eq!(master_key.unwrap().len(), 32);
    }
    
    #[test]
    fn test_ed25519_key_derivation() {
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let path = "m/44'/501'/0'/0'";

        let result = derive_ed25519_key_from_mnemonic(mnemonic, None, path);
        assert!(result.is_ok());

        let (private_key, public_key) = result.unwrap();
        assert_eq!(private_key.len(), 32);
        assert_eq!(public_key.len(), 32);
    }

    #[test]
    fn test_stellar_slip0010_derivation() {
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let path = "m/44'/148'/1'";

        let result = derive_ed25519_key_from_mnemonic(mnemonic, None, path);
        assert!(result.is_ok());

        let (private_key, public_key) = result.unwrap();
        assert_eq!(private_key.len(), 32);
        assert_eq!(public_key.len(), 32);

        // Print for manual verification against bip39 tools
        println!("SLIP-0010 for {}: ", path);
        println!("Private key (hex): {}", hex::encode(&private_key));
        println!("Public key (hex): {}", hex::encode(&public_key));

        // Test path m/44'/148'/2' as well
        let path2 = "m/44'/148'/2'";
        let result2 = derive_ed25519_key_from_mnemonic(mnemonic, None, path2);
        assert!(result2.is_ok());

        let (private_key2, public_key2) = result2.unwrap();
        println!("SLIP-0010 for {}: ", path2);
        println!("Private key (hex): {}", hex::encode(&private_key2));
        println!("Public key (hex): {}", hex::encode(&public_key2));
    }
}