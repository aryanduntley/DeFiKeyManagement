use clap::Args;
use anyhow::Result;
use crate::database::Database;

pub mod import;
pub mod derive;
pub mod list;
pub mod show;
pub mod export;
pub mod delete;
pub mod tag;
pub mod search;

pub use import::*;
pub use derive::*;
pub use list::*;
pub use show::*;
pub use export::*;
pub use delete::*;
pub use tag::*;
pub use search::*;

#[derive(Args)]
pub struct ImportArgs {
    /// Blockchain network (e.g., bitcoin, ethereum, solana)
    #[arg(short, long)]
    pub blockchain: String,
    
    /// BIP-39 mnemonic seed phrase
    #[arg(short, long)]
    pub mnemonic: Option<String>,
    
    /// Optional passphrase for mnemonic
    #[arg(short, long)]
    pub passphrase: Option<String>,
    
    /// Private key (alternative to mnemonic)
    #[arg(long)]
    pub private_key: Option<String>,
    
    /// Custom derivation path (overrides default)
    #[arg(long)]
    pub custom_path: Option<String>,
    
    /// Wallet label/name
    #[arg(short, long)]
    pub label: Option<String>,
}

#[derive(Args)]
pub struct DeriveArgs {
    /// BIP-39 mnemonic seed phrase
    #[arg(short, long)]
    pub mnemonic: String,
    
    /// Optional passphrase for mnemonic
    #[arg(short, long)]
    pub passphrase: Option<String>,
    
    /// Blockchain network
    #[arg(short, long)]
    pub blockchain: String,
    
    /// Account index (default: 0)
    #[arg(long, default_value = "0")]
    pub account: u32,
    
    /// Starting address index (default: 0)
    #[arg(long, default_value = "0")]
    pub index: u32,
    
    /// Number of addresses to derive (default: 1)
    #[arg(short, long, default_value = "1")]
    pub count: u32,
    
    /// Custom derivation path template
    #[arg(long)]
    pub custom_path: Option<String>,
}

#[derive(Args)]
pub struct ShowArgs {
    /// Wallet address to show
    #[arg(long)]
    pub address: Option<String>,
    
    /// Wallet label to show
    #[arg(short, long)]
    pub label: Option<String>,
    
    /// Include sensitive data (private key, mnemonic)
    #[arg(long)]
    pub include_sensitive: bool,
}

#[derive(Args)]
pub struct ExportArgs {
    /// Export format (json, csv)
    #[arg(short, long, default_value = "json")]
    pub format: String,
    
    /// Output file path
    #[arg(short, long)]
    pub output: Option<String>,
    
    /// Specific wallet address to export
    #[arg(long)]
    pub address: Option<String>,
    
    /// Specific wallet label to export
    #[arg(short, long)]
    pub label: Option<String>,
    
    /// Include sensitive data in export
    #[arg(long)]
    pub include_sensitive: bool,
}

#[derive(Args)]
pub struct DeleteArgs {
    /// Wallet address to delete
    #[arg(long)]
    pub address: Option<String>,
    
    /// Wallet label to delete
    #[arg(short, long)]
    pub label: Option<String>,
    
    /// Skip confirmation prompt
    #[arg(long)]
    pub force: bool,
}

#[derive(Args)]
pub struct TagArgs {
    /// Wallet address to update
    #[arg(long)]
    pub address: Option<String>,
    
    /// Current wallet label to update
    #[arg(long)]
    pub current_label: Option<String>,
    
    /// New label for the wallet
    #[arg(short, long)]
    pub label: String,
}

#[derive(Args)]
pub struct SearchArgs {
    /// Search term
    #[arg(short, long)]
    pub term: String,
    
    /// Search only in specific blockchain
    #[arg(short, long)]
    pub blockchain: Option<String>,
}

// Command handlers - to be implemented
pub fn handle_import(args: ImportArgs, db: &Database) -> Result<()> {
    import::execute(args, db)
}

pub fn handle_derive(args: DeriveArgs, db: &Database) -> Result<()> {
    derive::execute(args, db)
}

pub fn handle_list(db: &Database) -> Result<()> {
    list::execute(db)
}

pub fn handle_show(args: ShowArgs, db: &Database) -> Result<()> {
    show::execute(args, db)
}

pub fn handle_export(args: ExportArgs, db: &Database) -> Result<()> {
    export::execute(args, db)
}

pub fn handle_delete(args: DeleteArgs, db: &Database) -> Result<()> {
    delete::execute(args, db)
}

pub fn handle_tag(args: TagArgs, db: &Database) -> Result<()> {
    tag::execute(args, db)
}

pub fn handle_search(args: SearchArgs, db: &Database) -> Result<()> {
    search::execute(args, db)
}