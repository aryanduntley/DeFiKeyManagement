use anyhow::{Result, bail};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BipStandard {
    /// BIP-44: Multi-Account Hierarchy for Deterministic Wallets
    /// Purpose: 44' - Legacy addresses
    Bip44,

    /// BIP-49: Derivation scheme for P2SH-wrapped SegWit
    /// Purpose: 49' - P2SH-wrapped SegWit addresses (start with "3")
    Bip49,

    /// BIP-84: Derivation scheme for Native SegWit
    /// Purpose: 84' - Native SegWit addresses (start with "bc1")
    Bip84,

    /// BIP-141: Segregated Witness (SegWit) specification
    /// Purpose: Defines SegWit transaction format (not a derivation path)
    Bip141,

    /// BIP-85: Deterministic Entropy From BIP32 Keychains
    /// Purpose: 85' - For generating multiple mnemonics from a single seed
    Bip85,
}

impl BipStandard {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "44" | "bip44" | "bip-44" => Ok(Self::Bip44),
            "49" | "bip49" | "bip-49" => Ok(Self::Bip49),
            "84" | "bip84" | "bip-84" => Ok(Self::Bip84),
            "141" | "bip141" | "bip-141" => Ok(Self::Bip141),
            "85" | "bip85" | "bip-85" => Ok(Self::Bip85),
            _ => bail!("Unsupported BIP standard: {}. Supported: 44, 49, 84, 85, 141", s),
        }
    }

    pub fn get_purpose(&self) -> u32 {
        match self {
            Self::Bip44 => 44,
            Self::Bip49 => 49,
            Self::Bip84 => 84,
            Self::Bip141 => 141, // Note: BIP-141 is not typically used in derivation paths
            Self::Bip85 => 85,
        }
    }

    pub fn get_description(&self) -> &'static str {
        match self {
            Self::Bip44 => "Multi-Account Hierarchy (Legacy addresses)",
            Self::Bip49 => "P2SH-wrapped SegWit (addresses start with '3')",
            Self::Bip84 => "Native SegWit (addresses start with 'bc1')",
            Self::Bip141 => "Segregated Witness specification",
            Self::Bip85 => "Deterministic Entropy Generation",
        }
    }

    pub fn supports_derivation_path(&self) -> bool {
        match self {
            Self::Bip44 | Self::Bip49 | Self::Bip84 | Self::Bip85 => true,
            Self::Bip141 => false, // BIP-141 is a transaction format, not derivation
        }
    }

    pub fn get_derivation_path(&self, coin_type: u32, account: u32, change: u32, address_index: u32) -> String {
        if !self.supports_derivation_path() {
            return "N/A".to_string();
        }

        format!("m/{}'/{}'/{}'/{}/{}",
            self.get_purpose(),
            coin_type,
            account,
            change,
            address_index
        )
    }

    pub fn get_all_supported() -> Vec<Self> {
        vec![Self::Bip44, Self::Bip49, Self::Bip84, Self::Bip85]
    }
}

impl fmt::Display for BipStandard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BIP-{}", self.get_purpose())
    }
}

use crate::blockchain::SupportedBlockchain;

impl SupportedBlockchain {
    /// Returns the BIP standards supported by this blockchain
    pub fn get_supported_bips(&self) -> Vec<BipStandard> {
        match self {
            // Bitcoin supports all major BIPs
            Self::Bitcoin => vec![BipStandard::Bip44, BipStandard::Bip49, BipStandard::Bip84],

            // Litecoin supports the same BIPs as Bitcoin
            Self::Litecoin => vec![BipStandard::Bip44, BipStandard::Bip49, BipStandard::Bip84],

            // Most other blockchains primarily use BIP-44
            Self::Ethereum | Self::Stellar | Self::Solana | Self::XRP | Self::Cardano |
            Self::Tron | Self::Cronos | Self::Hedera | Self::Algorand | Self::Cosmos |
            Self::BinanceBNB | Self::Polygon | Self::Polkadot | Self::Sui | Self::Optimism |
            Self::IOTA | Self::XDC | Self::TON => vec![BipStandard::Bip44],
        }
    }

    /// Returns the default BIP standard for this blockchain
    pub fn get_default_bip(&self) -> BipStandard {
        match self {
            // Bitcoin and Litecoin default to Native SegWit (BIP-84) for modern wallets
            Self::Bitcoin | Self::Litecoin => BipStandard::Bip84,

            // All other blockchains use BIP-44
            _ => BipStandard::Bip44,
        }
    }

    /// Validates if the blockchain supports the given BIP standard
    pub fn supports_bip(&self, bip: BipStandard) -> bool {
        self.get_supported_bips().contains(&bip)
    }

    /// Gets the derivation path for this blockchain using the specified BIP standard
    pub fn get_bip_derivation_path(&self, bip: BipStandard, account: u32, address_index: u32) -> Result<String> {
        if !self.supports_bip(bip) {
            bail!("Blockchain {} does not support {}", self, bip);
        }

        if !bip.supports_derivation_path() {
            bail!("{} does not define a derivation path", bip);
        }

        let coin_type = self.get_coin_type().unwrap_or(0);

        // Most blockchains use change=0 for receiving addresses
        let change = 0;

        Ok(bip.get_derivation_path(coin_type, account, change, address_index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bip_parsing() {
        assert_eq!(BipStandard::from_str("44").unwrap(), BipStandard::Bip44);
        assert_eq!(BipStandard::from_str("bip84").unwrap(), BipStandard::Bip84);
        assert_eq!(BipStandard::from_str("BIP-49").unwrap(), BipStandard::Bip49);
        assert!(BipStandard::from_str("999").is_err());
    }

    #[test]
    fn test_bitcoin_bip_support() {
        let bitcoin = SupportedBlockchain::Bitcoin;
        assert!(bitcoin.supports_bip(BipStandard::Bip44));
        assert!(bitcoin.supports_bip(BipStandard::Bip49));
        assert!(bitcoin.supports_bip(BipStandard::Bip84));
        assert_eq!(bitcoin.get_default_bip(), BipStandard::Bip84);
    }

    #[test]
    fn test_ethereum_bip_support() {
        let ethereum = SupportedBlockchain::Ethereum;
        assert!(ethereum.supports_bip(BipStandard::Bip44));
        assert!(!ethereum.supports_bip(BipStandard::Bip84));
        assert_eq!(ethereum.get_default_bip(), BipStandard::Bip44);
    }

    #[test]
    fn test_derivation_paths() {
        let bitcoin = SupportedBlockchain::Bitcoin;

        // Test BIP-44 path for Bitcoin
        let bip44_path = bitcoin.get_bip_derivation_path(BipStandard::Bip44, 0, 0).unwrap();
        assert_eq!(bip44_path, "m/44'/0'/0'/0/0");

        // Test BIP-84 path for Bitcoin
        let bip84_path = bitcoin.get_bip_derivation_path(BipStandard::Bip84, 0, 0).unwrap();
        assert_eq!(bip84_path, "m/84'/0'/0'/0/0");

        // Test unsupported BIP for Ethereum
        let ethereum = SupportedBlockchain::Ethereum;
        assert!(ethereum.get_bip_derivation_path(BipStandard::Bip84, 0, 0).is_err());
    }
}