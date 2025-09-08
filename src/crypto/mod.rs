use anyhow::{Result, Context};
use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::Network;
use std::str::FromStr;

pub mod bip32;
pub mod bip39_utils;
pub mod ed25519_utils;

pub use bip32::*;
pub use bip39_utils::*;
pub use ed25519_utils::*;

pub fn validate_mnemonic(mnemonic: &str) -> Result<Mnemonic> {
    Mnemonic::from_str(mnemonic)
        .context("Invalid BIP-39 mnemonic phrase")
}

pub fn mnemonic_to_seed(mnemonic: &str, passphrase: Option<&str>) -> Result<Vec<u8>> {
    let mnemonic = validate_mnemonic(mnemonic)?;
    let passphrase = passphrase.unwrap_or("");
    let seed = mnemonic.to_seed(passphrase);
    Ok(seed.to_vec())
}

pub fn derive_master_key_secp256k1(seed: &[u8]) -> Result<Xpriv> {
    Xpriv::new_master(Network::Bitcoin, seed)
        .context("Failed to derive master private key")
}

pub fn derive_path_secp256k1(master_key: &Xpriv, path: &str) -> Result<Xpriv> {
    let derivation_path = DerivationPath::from_str(path)
        .context("Invalid derivation path")?;
    
    Ok(master_key.derive_priv(&bitcoin::secp256k1::Secp256k1::new(), &derivation_path)?)
}

pub fn validate_derivation_path(path: &str) -> Result<()> {
    DerivationPath::from_str(path)
        .context("Invalid derivation path format")?;
    Ok(())
}

pub fn validate_private_key_hex(private_key: &str) -> Result<Vec<u8>> {
    let key = if private_key.starts_with("0x") {
        &private_key[2..]
    } else {
        private_key
    };
    
    hex::decode(key).context("Invalid hexadecimal private key")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_mnemonic() {
        let valid_mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        assert!(validate_mnemonic(valid_mnemonic).is_ok());
        
        let invalid_mnemonic = "invalid mnemonic phrase";
        assert!(validate_mnemonic(invalid_mnemonic).is_err());
    }

    #[test]
    fn test_validate_derivation_path() {
        assert!(validate_derivation_path("m/44'/0'/0'/0/0").is_ok());
        assert!(validate_derivation_path("m/44'/60'/0'/0/0").is_ok());
        assert!(validate_derivation_path("invalid/path").is_err());
    }
}