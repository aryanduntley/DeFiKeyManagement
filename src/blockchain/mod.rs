use anyhow::{Result, bail};

pub mod bitcoin;
pub mod ethereum;
pub mod solana;
pub mod stellar;
pub mod common;

// Phase 1 blockchain handlers
pub mod xrp;
pub mod litecoin;
pub mod cardano;

pub use common::*;

#[derive(Debug, Clone)]
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
    Quant,
    TON,
}

impl SupportedBlockchain {
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
            "quant" | "qnt" => Ok(Self::Quant),
            "ton" => Ok(Self::TON),
            _ => bail!("Unsupported blockchain: {}", s),
        }
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
            Self::Quant => Some(1110),
            Self::TON => None, // Custom derivation
        }
    }
    
    pub fn get_default_derivation_path(&self, account: u32, address_index: u32) -> String {
        match self {
            Self::Bitcoin => format!("m/44'/0'/{}'/{}/{}", account, 0, address_index),
            Self::Ethereum => format!("m/44'/60'/{}'/{}/{}", account, 0, address_index),
            Self::Solana => format!("m/44'/501'/{}/{}'", account, address_index),
            Self::Stellar => format!("m/44'/148'/{}'", account),
            Self::XRP => format!("m/44'/144'/{}'/{}/{}", account, 0, address_index),
            Self::Cardano => format!("m/1852'/1815'/{}'/{}/{}", account, 0, address_index),
            Self::Tron => format!("m/44'/195'/{}'/{}/{}", account, 0, address_index),
            Self::Cronos => format!("m/44'/394'/{}'/{}/{}", account, 0, address_index),
            Self::Hedera => format!("m/44'/3030'/{}'/{}'/{}'", account, 0, address_index),
            Self::Algorand => format!("m/44'/283'/{}'/{}'/{}'", account, 0, address_index),
            Self::Cosmos => format!("m/44'/118'/{}'/{}/{}", account, 0, address_index),
            Self::BinanceBNB => format!("m/44'/714'/{}'/{}/{}", account, 0, address_index),
            Self::Litecoin => format!("m/44'/2'/{}'/{}/{}", account, 0, address_index),
            Self::Polygon => format!("m/44'/966'/{}'/{}/{}", account, 0, address_index),
            Self::Polkadot => format!("m/44'/354'/{}'/{}'/{}'", account, 0, address_index),
            Self::Sui => format!("m/44'/784'/{}'/{}'/{}'", account, 0, address_index),
            Self::Optimism => format!("m/44'/60'/{}'/{}/{}", account, 0, address_index),
            Self::IOTA => format!("m/44'/4218'/{}'/{}'/{}'", account, 0, address_index),
            Self::XDC => format!("m/44'/550'/{}'/{}/{}", account, 0, address_index),
            Self::Quant => format!("m/44'/1110'/{}'/{}/{}", account, 0, address_index),
            Self::TON => format!("m/44'/607'/{}'/{}'", account, address_index), // Custom
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
            Self::Quant => format!("https://etherscan.io/address/{}", address), // ERC-20
            Self::TON => format!("https://tonscan.org/address/{}", address),
        }
    }
}