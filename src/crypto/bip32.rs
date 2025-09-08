use anyhow::{Result, Context};
use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::Network;
use bip39::Mnemonic;
use std::str::FromStr;

pub fn derive_secp256k1_key_from_mnemonic(
    mnemonic: &str,
    passphrase: Option<&str>,
    derivation_path: &str,
) -> Result<(Vec<u8>, Vec<u8>)> {
    // Parse and validate mnemonic
    let mnemonic = Mnemonic::from_str(mnemonic)
        .context("Invalid BIP-39 mnemonic")?;
    
    // Generate seed from mnemonic + passphrase
    let seed = mnemonic.to_seed(passphrase.unwrap_or(""));
    
    // Create secp256k1 context
    let secp = Secp256k1::new();
    
    // Derive master private key from seed
    let master_key = Xpriv::new_master(Network::Bitcoin, &seed)
        .context("Failed to create master key")?;
    
    // Parse derivation path
    let path = DerivationPath::from_str(derivation_path)
        .context("Invalid derivation path")?;
    
    // Derive child key at specified path
    let derived_key = master_key.derive_priv(&secp, &path)
        .context("Failed to derive child key")?;
    
    // Get private key bytes
    let private_key = derived_key.private_key.secret_bytes().to_vec();
    
    // Derive public key from private key
    let public_key = derived_key.private_key.public_key(&secp);
    let public_key_bytes = public_key.serialize().to_vec();
    
    Ok((private_key, public_key_bytes))
}

pub fn private_key_to_public_key_secp256k1(private_key: &[u8]) -> Result<Vec<u8>> {
    use bitcoin::secp256k1::{SecretKey, PublicKey};
    
    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(private_key)
        .context("Invalid private key")?;
    
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);
    Ok(public_key.serialize().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_derive_key_from_mnemonic() {
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let path = "m/44'/0'/0'/0/0";
        
        let result = derive_secp256k1_key_from_mnemonic(mnemonic, None, path);
        assert!(result.is_ok());
        
        let (private_key, public_key) = result.unwrap();
        assert_eq!(private_key.len(), 32);
        assert_eq!(public_key.len(), 33); // Compressed public key
    }
    
    #[test]
    fn test_private_to_public_key() {
        let private_key = hex::decode("1111111111111111111111111111111111111111111111111111111111111111").unwrap();
        let public_key = private_key_to_public_key_secp256k1(&private_key);
        
        assert!(public_key.is_ok());
        assert_eq!(public_key.unwrap().len(), 33);
    }
}