use anyhow::{Result, Context};
use bip39::Mnemonic;
use std::str::FromStr;

pub fn validate_mnemonic_phrase(mnemonic: &str) -> Result<()> {
    Mnemonic::from_str(mnemonic)
        .context("Invalid BIP-39 mnemonic phrase")?;
    Ok(())
}

pub fn mnemonic_to_entropy(mnemonic: &str) -> Result<Vec<u8>> {
    let mnemonic = Mnemonic::from_str(mnemonic)
        .context("Invalid BIP-39 mnemonic phrase")?;
    Ok(mnemonic.to_entropy())
}

pub fn entropy_to_mnemonic(entropy: &[u8]) -> Result<String> {
    let mnemonic = Mnemonic::from_entropy(entropy)
        .context("Invalid entropy for mnemonic generation")?;
    Ok(mnemonic.to_string())
}

pub fn generate_seed_from_mnemonic(mnemonic: &str, passphrase: Option<&str>) -> Result<Vec<u8>> {
    let mnemonic = Mnemonic::from_str(mnemonic)
        .context("Invalid BIP-39 mnemonic phrase")?;
    
    let seed = mnemonic.to_seed(passphrase.unwrap_or(""));
    Ok(seed.to_vec())
}

pub fn is_valid_mnemonic_length(word_count: usize) -> bool {
    matches!(word_count, 12 | 15 | 18 | 21 | 24)
}

pub fn get_mnemonic_word_count(mnemonic: &str) -> usize {
    mnemonic.split_whitespace().count()
}

pub fn normalize_mnemonic(mnemonic: &str) -> String {
    mnemonic
        .split_whitespace()
        .map(|word| word.trim().to_lowercase())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_mnemonic() {
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        assert!(validate_mnemonic_phrase(mnemonic).is_ok());
    }

    #[test]
    fn test_invalid_mnemonic() {
        let mnemonic = "invalid mnemonic phrase that does not follow bip39";
        assert!(validate_mnemonic_phrase(mnemonic).is_err());
    }

    #[test]
    fn test_mnemonic_word_count() {
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        assert_eq!(get_mnemonic_word_count(mnemonic), 12);
        assert!(is_valid_mnemonic_length(12));
    }

    #[test]
    fn test_normalize_mnemonic() {
        let mnemonic = "  ABANDON   abandon  About  ";
        let normalized = normalize_mnemonic(mnemonic);
        assert_eq!(normalized, "abandon abandon about");
    }

    #[test]
    fn test_seed_generation() {
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        
        let seed_no_passphrase = generate_seed_from_mnemonic(mnemonic, None).unwrap();
        let seed_with_passphrase = generate_seed_from_mnemonic(mnemonic, Some("test")).unwrap();
        
        assert_eq!(seed_no_passphrase.len(), 64);
        assert_eq!(seed_with_passphrase.len(), 64);
        assert_ne!(seed_no_passphrase, seed_with_passphrase);
    }
}