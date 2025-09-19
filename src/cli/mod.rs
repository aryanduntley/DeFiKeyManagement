use clap::Args;
use anyhow::Result;
use crate::database::Database;

// New hierarchical commands
pub mod hierarchy;

// Legacy commands (TEMPORARILY DISABLED - TO BE REPLACED)
// pub mod import;
// pub mod derive;
// pub mod list;
// pub mod show;
// pub mod export;
// pub mod delete;
// pub mod tag;
// pub mod search;
// pub mod import_multi;
// pub mod list_groups;
// pub mod show_group;
// pub mod derive_multi;
// pub mod rename_group;

// New hierarchical commands
pub use hierarchy::*;

// Legacy commands (TEMPORARILY DISABLED)
// pub use import::*;
// pub use derive::*;
// pub use list::*;
// pub use show::*;
// pub use export::*;
// pub use delete::*;
// pub use tag::*;
// pub use search::*;
// pub use import_multi::*;
// pub use list_groups::*;
// pub use show_group::*;
// pub use derive_multi::*;
// pub use rename_group::*;

// TEMPORARILY DISABLED - TO BE REPLACED
// #[derive(Args)]
// pub struct ImportArgs {
//     /// Blockchain network (e.g., bitcoin, ethereum, solana)
//     #[arg(short, long)]
//     pub blockchain: String,

//     /// BIP-39 mnemonic seed phrase
//     #[arg(short, long)]
//     pub mnemonic: Option<String>,

//     /// Optional passphrase for mnemonic
//     #[arg(short, long)]
//     pub passphrase: Option<String>,

//     /// Private key (alternative to mnemonic)
//     #[arg(long)]
//     pub private_key: Option<String>,

//     /// Custom derivation path (overrides default)
//     #[arg(long)]
//     pub custom_path: Option<String>,

//     /// Wallet label/name
//     #[arg(short, long)]
//     pub label: Option<String>,
// }

// TEMPORARILY DISABLED - TO BE REPLACED
// #[derive(Args)]
// pub struct ImportMultiArgs {
//     /// BIP-39 mnemonic seed phrase
//     #[arg(short, long)]
//     pub mnemonic: String,

//     /// Wallet group name
//     #[arg(short, long)]
//     pub group_name: String,

//     /// Optional group description
//     #[arg(short, long)]
//     pub description: Option<String>,

//     /// Comma-separated list of blockchains (e.g., "bitcoin,ethereum,solana")
//     /// If not specified, defaults to: bitcoin,ethereum,solana,polygon,binance
//     #[arg(short, long)]
//     pub blockchains: Option<String>,

//     /// Optional passphrase for mnemonic
//     #[arg(short, long)]
//     pub passphrase: Option<String>,

//     /// Account index (default: 0)
//     #[arg(long)]
//     pub account: Option<u32>,

//     /// Address index (default: 0)
//     #[arg(long)]
//     pub address_index: Option<u32>,
// }

// TEMPORARILY DISABLED - TO BE REPLACED
// #[derive(Args)]
// pub struct ShowGroupArgs {
//     /// Wallet group name to display
//     pub group_name: String,

//     /// Include sensitive data (private keys, mnemonics)
//     #[arg(long)]
//     pub include_sensitive: bool,
// }

// TEMPORARILY DISABLED - TO BE REPLACED
// #[derive(Args)]
// pub struct DeriveMultiArgs {
//     /// Wallet group name to extend
//     #[arg(short, long)]
//     pub group_name: String,

//     /// Comma-separated list of blockchains to add (e.g., "cardano,polkadot,sui")
//     #[arg(short, long)]
//     pub blockchains: String,

//     /// BIP-39 mnemonic seed phrase (required to verify group ownership)
//     #[arg(short, long)]
//     pub mnemonic: Option<String>,

//     /// Optional passphrase for mnemonic
//     #[arg(short, long)]
//     pub passphrase: Option<String>,

//     /// Account index (default: 0)
//     #[arg(long)]
//     pub account: Option<u32>,

//     /// Address index (default: 0)
//     #[arg(long)]
//     pub address_index: Option<u32>,
// }

// TEMPORARILY DISABLED - TO BE REPLACED
// #[derive(Args)]
// pub struct RenameGroupArgs {
//     /// Current group name
//     #[arg(short, long)]
//     pub old_name: String,

//     /// New group name
//     #[arg(short, long)]
//     pub new_name: String,

//     /// Skip confirmation prompt
//     #[arg(long)]
//     pub force: bool,
// }

// TEMPORARILY DISABLED - TO BE REPLACED
// #[derive(Args)]
// pub struct DeriveArgs {
//     /// BIP-39 mnemonic seed phrase
//     #[arg(short, long)]
//     pub mnemonic: String,

//     /// Optional passphrase for mnemonic
//     #[arg(short, long)]
//     pub passphrase: Option<String>,

//     /// Blockchain network
//     #[arg(short, long)]
//     pub blockchain: String,

//     /// Account index (default: 0)
//     #[arg(long, default_value = "0")]
//     pub account: u32,

//     /// Starting address index (default: 0)
//     #[arg(long, default_value = "0")]
//     pub index: u32,

//     /// Number of addresses to derive (default: 1)
//     #[arg(short, long, default_value = "1")]
//     pub count: u32,

//     /// Custom derivation path template
//     #[arg(long)]
//     pub custom_path: Option<String>,
// }

// TEMPORARILY DISABLED - TO BE REPLACED
// #[derive(Args)]
// pub struct ShowArgs {
//     /// Wallet address to show
//     #[arg(long)]
//     pub address: Option<String>,

//     /// Wallet label to show
//     #[arg(short, long)]
//     pub label: Option<String>,

//     /// Include sensitive data (private key, mnemonic)
//     #[arg(long)]
//     pub include_sensitive: bool,
// }

// #[derive(Args)]
// pub struct ExportArgs {
//     /// Export format (json, csv)
//     #[arg(short, long, default_value = "json")]
//     pub format: String,

//     /// Output file path
//     #[arg(short, long)]
//     pub output: Option<String>,

//     /// Specific wallet address to export
//     #[arg(long)]
//     pub address: Option<String>,

//     /// Specific wallet label to export
//     #[arg(short, long)]
//     pub label: Option<String>,

//     /// Include sensitive data in export
//     #[arg(long)]
//     pub include_sensitive: bool,
// }

// #[derive(Args)]
// pub struct DeleteArgs {
//     /// Wallet address to delete
//     #[arg(long)]
//     pub address: Option<String>,

//     /// Wallet label to delete
//     #[arg(short, long)]
//     pub label: Option<String>,

//     /// Skip confirmation prompt
//     #[arg(long)]
//     pub force: bool,
// }

// #[derive(Args)]
// pub struct TagArgs {
//     /// Wallet address to update
//     #[arg(long)]
//     pub address: Option<String>,

//     /// Current wallet label to update
//     #[arg(long)]
//     pub current_label: Option<String>,

//     /// New label for the wallet
//     #[arg(short, long)]
//     pub label: String,
// }

// #[derive(Args)]
// pub struct SearchArgs {
//     /// Search term
//     #[arg(short, long)]
//     pub term: String,

//     /// Search only in specific blockchain
//     #[arg(short, long)]
//     pub blockchain: Option<String>,
// }

// Hierarchical command handlers
pub fn handle_create_master(args: CreateMasterArgs, db: &Database) -> Result<()> {
    hierarchy::master_account::create_master::execute(args, db)
}

pub fn handle_list_accounts(args: ListAccountsArgs, db: &Database) -> Result<()> {
    hierarchy::master_account::list_accounts::execute(args, db)
}

pub fn handle_create_wallet_group(args: CreateWalletGroupArgs, db: &Database) -> Result<()> {
    hierarchy::wallet_group::create_wallet_group::execute(args, db)
}

pub fn handle_add_blockchain(args: AddBlockchainArgs, db: &Database) -> Result<()> {
    hierarchy::blockchain::add_blockchain::execute(args, db)
}

pub fn handle_list_wallet_groups(args: ListWalletGroupsArgs, db: &Database) -> Result<()> {
    hierarchy::wallet_group::list_wallet_groups::execute(args, db)
}

pub fn handle_show_wallet_group(args: ShowWalletGroupArgs, db: &Database) -> Result<()> {
    hierarchy::wallet_group::show_wallet_group::execute(args, db)
}

// Legacy command handlers (TEMPORARILY DISABLED)
// pub fn handle_import(args: ImportArgs, db: &Database) -> Result<()> {
//     import::execute(args, db)
// }

// pub fn handle_derive(args: DeriveArgs, db: &Database) -> Result<()> {
//     derive::execute(args, db)
// }

// pub fn handle_list(db: &Database) -> Result<()> {
//     list::execute(db)
// }

// pub fn handle_show(args: ShowArgs, db: &Database) -> Result<()> {
//     show::execute(args, db)
// }

// pub fn handle_export(args: ExportArgs, db: &Database) -> Result<()> {
//     export::execute(args, db)
// }

// pub fn handle_delete(args: DeleteArgs, db: &Database) -> Result<()> {
//     delete::execute(args, db)
// }

// pub fn handle_tag(args: TagArgs, db: &Database) -> Result<()> {
//     tag::execute(args, db)
// }

// pub fn handle_search(args: SearchArgs, db: &Database) -> Result<()> {
//     search::execute(args, db)
// }

// pub fn handle_import_multi(args: ImportMultiArgs, db: &Database) -> Result<()> {
//     import_multi::execute(args, db)
// }

// pub fn handle_list_groups(db: &Database) -> Result<()> {
//     list_groups::execute(db)
// }

// pub fn handle_show_group(args: ShowGroupArgs, db: &Database) -> Result<()> {
//     show_group::execute(args, db)
// }

// pub fn handle_derive_multi(args: DeriveMultiArgs, db: &Database) -> Result<()> {
//     derive_multi::execute(args, db)
// }

// pub fn handle_rename_group(args: RenameGroupArgs, db: &Database) -> Result<()> {
//     rename_group::execute(args, db)
// }