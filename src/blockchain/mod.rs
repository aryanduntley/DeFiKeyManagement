use anyhow::{Result, bail};

pub mod bitcoin;
pub mod ethereum;
pub mod solana;
pub mod stellar;
pub mod common;
pub mod bip_standards;

// Phase 1 blockchain handlers
pub mod xrp;
pub mod litecoin;
pub mod cardano;

// Phase 2 blockchain handlers
pub mod tron;
pub mod polygon;
pub mod optimism;

// Phase 3 blockchain handlers
pub mod cronos;
pub mod binance;
pub mod cosmos;

// Phase 4 blockchain handlers
pub mod algorand;
pub mod hedera;
pub mod polkadot;

// Phase 5 blockchain handlers
pub mod sui;
pub mod iota;
pub mod ton;

// Phase 6 blockchain handlers
pub mod xdc;

pub use common::*;
pub use bip_standards::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SupportedBlockchain {
    Bitcoin,
    Ethereum,
    Solana,
    Stellar,
    XRP,
    Cardano,
    Tron,
    Cronos,
    Hedera,
    Algorand,
    Cosmos,
    BinanceBNB,
    Litecoin,
    Polygon,
    Polkadot,
    Sui,
    Optimism,
    IOTA,
    XDC,
    TON,
}

impl std::fmt::Display for SupportedBlockchain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bitcoin => write!(f, "bitcoin"),
            Self::Ethereum => write!(f, "ethereum"),
            Self::Solana => write!(f, "solana"),
            Self::Stellar => write!(f, "stellar"),
            Self::XRP => write!(f, "xrp"),
            Self::Cardano => write!(f, "cardano"),
            Self::Tron => write!(f, "tron"),
            Self::Cronos => write!(f, "cronos"),
            Self::Hedera => write!(f, "hedera"),
            Self::Algorand => write!(f, "algorand"),
            Self::Cosmos => write!(f, "cosmos"),
            Self::BinanceBNB => write!(f, "binance"),
            Self::Litecoin => write!(f, "litecoin"),
            Self::Polygon => write!(f, "polygon"),
            Self::Polkadot => write!(f, "polkadot"),
            Self::Sui => write!(f, "sui"),
            Self::Optimism => write!(f, "optimism"),
            Self::IOTA => write!(f, "iota"),
            Self::XDC => write!(f, "xdc"),
            Self::TON => write!(f, "ton"),
        }
    }
}

impl SupportedBlockchain {
    /// Returns the maximum hierarchy level supported by this blockchain
    /// Level 3: Account/Wallet Group/Base Wallet only (Stellar)
    /// Level 4: Account/Wallet Group/Base Wallet/Address Group only (Solana)
    /// Level 5: Full hierarchy including subwallets (most blockchains)
    pub fn max_hierarchy_level(&self) -> u8 {
        match self {
            Self::Stellar => 3,  // m/44'/148'/0' - No Address Groups or Subwallets
            Self::Solana => 4,   // m/44'/501'/0'/0' - No Subwallets
            _ => 5,              // m/44'/xxx'/0'/0/0 - Full hierarchy
        }
    }

    /// Returns true if this blockchain supports Address Groups (level 4+)
    pub fn supports_address_groups(&self) -> bool {
        self.max_hierarchy_level() >= 4
    }

    /// Returns true if this blockchain supports Subwallets (level 5)
    pub fn supports_subwallets(&self) -> bool {
        self.max_hierarchy_level() >= 5
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "bitcoin" | "btc" => Ok(Self::Bitcoin),
            "ethereum" | "eth" => Ok(Self::Ethereum),
            "solana" | "sol" => Ok(Self::Solana),
            "stellar" | "xlm" => Ok(Self::Stellar),
            "xrp" | "ripple" => Ok(Self::XRP),
            "cardano" | "ada" => Ok(Self::Cardano),
            "tron" | "trx" => Ok(Self::Tron),
            "cronos" | "cro" => Ok(Self::Cronos),
            "hedera" | "hbar" => Ok(Self::Hedera),
            "algorand" | "algo" => Ok(Self::Algorand),
            "cosmos" | "atom" => Ok(Self::Cosmos),
            "binance" | "bnb" => Ok(Self::BinanceBNB),
            "litecoin" | "ltc" => Ok(Self::Litecoin),
            "polygon" | "matic" => Ok(Self::Polygon),
            "polkadot" | "dot" => Ok(Self::Polkadot),
            "sui" => Ok(Self::Sui),
            "optimism" | "op" => Ok(Self::Optimism),
            "iota" => Ok(Self::IOTA),
            "xdc" => Ok(Self::XDC),
            "ton" => Ok(Self::TON),
            _ => bail!("Unsupported blockchain: {}", s),
        }
    }

    /// Validates multiple blockchain names and returns the list of supported ones
    /// Returns an error with details about unsupported blockchains if any are found
    pub fn validate_blockchains(blockchain_names: &[String]) -> Result<Vec<Self>> {
        let mut supported = Vec::new();
        let mut unsupported = Vec::new();

        for name in blockchain_names {
            match Self::from_str(name) {
                Ok(blockchain) => supported.push(blockchain),
                Err(_) => unsupported.push(name.clone()),
            }
        }

        if !unsupported.is_empty() {
            let supported_list = Self::get_supported_blockchain_names().join(", ");
            bail!(
                "Unsupported blockchain(s): {}. Supported blockchains are: {}",
                unsupported.join(", "),
                supported_list
            );
        }

        Ok(supported)
    }

    /// Returns a list of all supported blockchain names for help/error messages
    pub fn get_supported_blockchain_names() -> Vec<String> {
        vec![
            "bitcoin".to_string(), "btc".to_string(),
            "ethereum".to_string(), "eth".to_string(),
            "solana".to_string(), "sol".to_string(),
            "stellar".to_string(), "xlm".to_string(),
            "xrp".to_string(), "ripple".to_string(),
            "cardano".to_string(), "ada".to_string(),
            "tron".to_string(), "trx".to_string(),
            "cronos".to_string(), "cro".to_string(),
            "hedera".to_string(), "hbar".to_string(),
            "algorand".to_string(), "algo".to_string(),
            "cosmos".to_string(), "atom".to_string(),
            "binance".to_string(), "bnb".to_string(),
            "litecoin".to_string(), "ltc".to_string(),
            "polygon".to_string(), "matic".to_string(),
            "polkadot".to_string(), "dot".to_string(),
            "sui".to_string(),
            "optimism".to_string(), "op".to_string(),
            "iota".to_string(),
            "xdc".to_string(),
            "ton".to_string(),
        ]
    }
    
    pub fn get_coin_type(&self) -> Option<u32> {
        match self {
            Self::Bitcoin => Some(0),
            Self::Ethereum => Some(60),
            Self::Solana => Some(501),
            Self::Stellar => Some(148),
            Self::XRP => Some(144),
            Self::Cardano => Some(1815),
            Self::Tron => Some(195),
            Self::Cronos => Some(394),
            Self::Hedera => Some(3030),
            Self::Algorand => Some(283),
            Self::Cosmos => Some(118),
            Self::BinanceBNB => Some(714),
            Self::Litecoin => Some(2),
            Self::Polygon => Some(966),
            Self::Polkadot => Some(354),
            Self::Sui => Some(784),
            Self::Optimism => Some(60), // Uses ETH derivation
            Self::IOTA => Some(4218),
            Self::XDC => Some(550),
            Self::TON => None, // Custom derivation
        }
    }
    
    /// Customizes derivation paths for blockchains that follow modified BIP standards
    pub fn customize_derivation_path(&self, standard_path: String) -> String {
        match self {
            Self::Stellar => {
                // Stellar uses BIP-44 but truncates to 3 levels due to ed25519 hardened-only requirement
                // Standard: m/44'/148'/account'/change/address_index
                // Stellar:  m/44'/148'/account'

                // Extract the account index from the standard path and create Stellar-specific path
                // Expected input: m/44'/148'/0'/0/0
                // Expected output: m/44'/148'/0'
                if standard_path.starts_with("m/44'/148'/") {
                    let parts: Vec<&str> = standard_path.split('/').collect();
                    if parts.len() >= 4 {
                        let account_part = parts[3]; // This should be the account index
                        // Remove trailing "'" if present, then add it back to ensure hardened
                        let account_num = account_part.trim_end_matches('\'');
                        return format!("m/44'/148'/{}'" , account_num);
                    }
                }

                // Fallback to original if parsing fails
                standard_path
            },
            Self::Solana => {
                // Solana uses BIP-44 but truncates to 4 levels
                // Standard: m/44'/501'/account'/change/address_index
                // Solana:   m/44'/501'/account'/0'

                // Extract the account index from the standard path and create Solana-specific path
                // Expected input: m/44'/501'/0'/0/0
                // Expected output: m/44'/501'/0'/0'
                if standard_path.starts_with("m/44'/501'/") {
                    let parts: Vec<&str> = standard_path.split('/').collect();
                    if parts.len() >= 5 {
                        let account_part = parts[3]; // This should be the account index
                        let change_part = parts[4];  // This should be the change index
                        // Remove trailing "'" if present, then add it back to ensure hardened
                        let account_num = account_part.trim_end_matches('\'');
                        let change_num = change_part.trim_end_matches('\'');
                        return format!("m/44'/501'/{}'/{}'", account_num, change_num);
                    }
                }

                // Fallback to original if parsing fails
                standard_path
            },
            _ => standard_path, // No customization needed
        }
    }

    pub fn get_default_derivation_path(&self, account: u32, address_index: u32) -> String {
        self.get_derivation_path_with_role(account, address_index, None)
    }

    pub fn get_derivation_path_with_role(&self, account: u32, address_index: u32, role: Option<u32>) -> String {
        // For blockchains that need path customization, apply it directly
        match self {
            Self::Stellar => {
                // Stellar uses 3-level hardened path: m/44'/148'/account'
                format!("m/44'/148'/{}'" , account)
            },
            Self::Solana => {
                // Solana uses 4-level hardened path: m/44'/501'/account'/0'
                format!("m/44'/501'/{}'/{}'", account, 0)
            },
            Self::Cardano => {
                let cardano_role = role.unwrap_or(0);
                // CIP-1852 standard: m/1852'/1815'/account'/role/address_index
                format!("m/1852'/1815'/{}'/{}/{}", account, cardano_role, address_index)
            },
            _ => {
                // For other blockchains, use standard BIP derivation
                let default_bip = self.get_default_bip();
                self.get_bip_derivation_path(default_bip, account, address_index)
                    .unwrap_or_else(|_| {
                        // Fallback to hardcoded paths for blockchains that don't support BIPs
                        match self {
                            Self::Cardano => {
                                let cardano_role = role.unwrap_or(0);
                                format!("m/1852'/1815'/{}'/{}/{}", account, cardano_role, address_index)
                            },
                            Self::Hedera => format!("m/44'/3030'/{}'/{}'/{}'", account, 0, address_index),
                            Self::Algorand => format!("m/44'/283'/{}'/{}'/{}'", account, 0, address_index),
                            Self::Polkadot => format!("m/44'/354'/{}'/{}'/{}'", account, 0, address_index),
                            Self::Sui => format!("m/44'/784'/{}'/{}'/{}'", account, 0, address_index),
                            Self::IOTA => format!("m/44'/4218'/{}'/{}'/{}'", account, 0, address_index),
                            Self::TON => format!("m/44'/607'/{}'/{}'", account, address_index),
                            _ => format!("m/44'/0'/{}'/{}/{}", account, 0, address_index),
                        }
                    })
            }
        }
    }
    
    pub fn uses_ed25519(&self) -> bool {
        matches!(self, 
            Self::Solana | Self::Stellar | Self::Cardano | 
            Self::Hedera | Self::Algorand | Self::Polkadot | 
            Self::Sui | Self::IOTA | Self::TON
        )
    }
    
    pub fn get_explorer_url(&self, address: &str) -> String {
        match self {
            Self::Bitcoin => format!("https://blockstream.info/address/{}", address),
            Self::Ethereum => format!("https://etherscan.io/address/{}", address),
            Self::Solana => format!("https://explorer.solana.com/address/{}", address),
            Self::Stellar => format!("https://stellar.expert/explorer/public/account/{}", address),
            Self::XRP => format!("https://xrpscan.com/account/{}", address),
            Self::Cardano => format!("https://cardanoscan.io/address/{}", address),
            Self::Tron => format!("https://tronscan.org/#/address/{}", address),
            Self::Cronos => format!("https://cronoscan.com/address/{}", address),
            Self::Hedera => format!("https://hashscan.io/mainnet/account/{}", address),
            Self::Algorand => format!("https://algoexplorer.io/address/{}", address),
            Self::Cosmos => format!("https://www.mintscan.io/cosmos/account/{}", address),
            Self::BinanceBNB => format!("https://bscscan.com/address/{}", address),
            Self::Litecoin => format!("https://blockchair.com/litecoin/address/{}", address),
            Self::Polygon => format!("https://polygonscan.com/address/{}", address),
            Self::Polkadot => format!("https://polkadot.subscan.io/account/{}", address),
            Self::Sui => format!("https://suiexplorer.com/address/{}", address),
            Self::Optimism => format!("https://optimistic.etherscan.io/address/{}", address),
            Self::IOTA => format!("https://explorer.iota.org/mainnet/addr/{}", address),
            Self::XDC => format!("https://explorer.xinfin.network/address/{}", address),
            Self::TON => format!("https://tonscan.org/address/{}", address),
        }
    }
}