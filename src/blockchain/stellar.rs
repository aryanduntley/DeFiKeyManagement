use anyhow::{Result, Context};
use crate::blockchain::{BlockchainHandler, WalletKeys, SupportedBlockchain};
use crate::crypto::ed25519_utils::{derive_ed25519_key_from_mnemonic, private_key_to_public_key_ed25519};
use ed25519_dalek::{SigningKey, VerifyingKey};

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
        
        // Derive ed25519 keys using SLIP-0010
        let (private_key_bytes, public_key_bytes) = derive_ed25519_key_from_mnemonic(
            mnemonic,
            passphrase,
            &derivation_path,
        )?;
        
        // Generate Stellar address from public key
        let address = self.public_key_to_address(&public_key_bytes)?;
        
        Ok(WalletKeys {
            private_key: hex::encode(&private_key_bytes),
            public_key: hex::encode(&public_key_bytes),
            address,
            derivation_path,
        })
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
        
        Ok(WalletKeys {
            private_key: hex::encode(&private_key_bytes),
            public_key: hex::encode(&public_key_bytes),
            address,
            derivation_path: "imported".to_string(),
        })
    }
    
    fn validate_address(&self, address: &str) -> bool {
        // Stellar addresses start with 'G' and are 56 characters long
        address.len() == 56 && address.starts_with('G') && self.is_valid_base32(address)
    }
    
    fn get_blockchain_name(&self) -> &'static str {
        "stellar"
    }
}

impl StellarHandler {
    fn public_key_to_address(&self, public_key_bytes: &[u8]) -> Result<String> {
        if public_key_bytes.len() != 32 {
            return Err(anyhow::anyhow!("Invalid public key length for Stellar"));
        }
        
        // Stellar address format: 'G' + base32(public_key + checksum)
        let address_bytes = self.add_stellar_checksum(public_key_bytes, 6 << 3)?; // Account ID version
        let base32_encoded = self.base32_encode(&address_bytes);
        
        Ok(format!("G{}", base32_encoded))
    }
    
    /// Add Stellar-style CRC16 checksum to data
    fn add_stellar_checksum(&self, data: &[u8], version: u16) -> Result<Vec<u8>> {
        let mut payload = Vec::new();
        payload.extend_from_slice(data);
        
        // Add version bytes
        payload.extend_from_slice(&version.to_le_bytes());
        
        // Calculate CRC16 checksum
        let checksum = self.crc16_xmodem(&payload);
        payload.extend_from_slice(&checksum.to_le_bytes());
        
        Ok(payload)
    }
    
    /// CRC16-XMODEM checksum used by Stellar
    fn crc16_xmodem(&self, data: &[u8]) -> u16 {
        let mut crc: u16 = 0x0000;
        
        for &byte in data {
            crc ^= (byte as u16) << 8;
            for _ in 0..8 {
                if crc & 0x8000 != 0 {
                    crc = (crc << 1) ^ 0x1021;
                } else {
                    crc <<= 1;
                }
            }
        }
        
        crc
    }
    
    /// Stellar uses a custom base32 encoding
    fn base32_encode(&self, data: &[u8]) -> String {
        const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
        
        let mut result = String::new();
        let mut buffer = 0u64;
        let mut buffer_len = 0;
        
        for &byte in data {
            buffer = (buffer << 8) | byte as u64;
            buffer_len += 8;
            
            while buffer_len >= 5 {
                let index = ((buffer >> (buffer_len - 5)) & 0x1f) as usize;
                result.push(ALPHABET[index] as char);
                buffer_len -= 5;
            }
        }
        
        if buffer_len > 0 {
            let index = ((buffer << (5 - buffer_len)) & 0x1f) as usize;
            result.push(ALPHABET[index] as char);
        }
        
        result
    }
    
    /// Check if string contains only valid base32 characters
    fn is_valid_base32(&self, s: &str) -> bool {
        s.chars().all(|c| {
            c.is_ascii_uppercase() || c.is_ascii_digit() && "234567".contains(c)
        })
    }
    
    /// Convert Stellar address back to public key bytes
    pub fn address_to_public_key(&self, address: &str) -> Result<Vec<u8>> {
        if !self.validate_address(address) {
            return Err(anyhow::anyhow!("Invalid Stellar address format"));
        }
        
        let base32_part = &address[1..]; // Remove 'G' prefix
        let decoded = self.base32_decode(base32_part)?;
        
        if decoded.len() < 32 {
            return Err(anyhow::anyhow!("Decoded address too short"));
        }
        
        // Extract public key (first 32 bytes)
        Ok(decoded[0..32].to_vec())
    }
    
    /// Stellar base32 decoder
    fn base32_decode(&self, s: &str) -> Result<Vec<u8>> {
        const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
        
        let mut result = Vec::new();
        let mut buffer = 0u64;
        let mut buffer_len = 0;
        
        for c in s.chars() {
            let value = ALPHABET.iter().position(|&x| x as char == c)
                .ok_or_else(|| anyhow::anyhow!("Invalid base32 character: {}", c))? as u64;
            
            buffer = (buffer << 5) | value;
            buffer_len += 5;
            
            if buffer_len >= 8 {
                result.push(((buffer >> (buffer_len - 8)) & 0xff) as u8);
                buffer_len -= 8;
            }
        }
        
        Ok(result)
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
    fn test_base32_encoding() {
        let handler = StellarHandler::new();
        
        let data = b"hello world";
        let encoded = handler.base32_encode(data);
        let decoded = handler.base32_decode(&encoded).unwrap();
        
        assert_eq!(decoded, data);
    }
    
    #[test]
    fn test_crc16_checksum() {
        let handler = StellarHandler::new();
        
        let data = b"test data";
        let checksum1 = handler.crc16_xmodem(data);
        let checksum2 = handler.crc16_xmodem(data);
        
        // Same data should produce same checksum
        assert_eq!(checksum1, checksum2);
        
        // Different data should produce different checksum
        let checksum3 = handler.crc16_xmodem(b"different data");
        assert_ne!(checksum1, checksum3);
    }
    
    #[test]
    fn test_address_roundtrip() {
        let handler = StellarHandler::new();
        let private_key = "2222222222222222222222222222222222222222222222222222222222222222";
        
        let keys = handler.derive_from_private_key(private_key).unwrap();
        let public_key_from_address = handler.address_to_public_key(&keys.address).unwrap();
        let original_public_key = hex::decode(&keys.public_key).unwrap();
        
        assert_eq!(public_key_from_address, original_public_key);
    }
}