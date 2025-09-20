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

pub fn handle_show_account(args: ShowAccountArgs, db: &Database) -> Result<()> {
    hierarchy::master_account::show_account::execute(args, db)
}

pub fn handle_create_wallet_group(args: CreateWalletGroupArgs, db: &Database) -> Result<()> {
    hierarchy::wallet_group::create_wallet_group::execute(args, db)
}

pub fn handle_add_wallet(args: AddWalletArgs, db: &Database) -> Result<()> {
    hierarchy::blockchain::add_wallet::execute(args, db)
}

pub fn handle_list_wallet_groups(args: ListWalletGroupsArgs, db: &Database) -> Result<()> {
    hierarchy::wallet_group::list_wallet_groups::execute(args, db)
}

pub fn handle_show_wallet_group(args: ShowWalletGroupArgs, db: &Database) -> Result<()> {
    hierarchy::wallet_group::show_wallet_group::execute(args, db)
}

pub fn handle_show_address_group(args: ShowAddressGroupArgs, db: &Database) -> Result<()> {
    hierarchy::address_group::show_address_group::execute(args, db)
}

pub fn handle_add_standalone_wallet(args: AddStandaloneWalletArgs, db: &Database) -> Result<()> {
    hierarchy::standalone::add_standalone_wallet::execute(args, db)
}

pub fn handle_add_address_group(args: AddAddressGroupArgs, db: &Database) -> Result<()> {
    hierarchy::address_group::add_address_group::execute(args, db)
}

pub fn handle_list_wallets(args: ListWalletsArgs, db: &Database) -> Result<()> {
    hierarchy::wallet_group::list_wallets::execute(args, db)
}

pub fn handle_list_standalone_wallets(args: ListStandaloneWalletsArgs, db: &Database) -> Result<()> {
    hierarchy::standalone::list_standalone_wallets::execute(args, db)
}

pub fn handle_list_address_groups(args: ListAddressGroupsArgs, db: &Database) -> Result<()> {
    hierarchy::address_group::list_address_groups::execute(args, db)
}

pub fn handle_add_subwallet(args: AddSubwalletArgs, db: &Database) -> Result<()> {
    hierarchy::subwallet::add_subwallet::execute(args, db)
}

pub fn handle_list_subwallets(args: ListSubwalletsArgs, db: &Database) -> Result<()> {
    hierarchy::subwallet::list_subwallets::execute(args, db)
}

pub fn handle_list_cryptocurrencies(args: ListCryptocurrenciesArgs, db: &Database) -> Result<()> {
    hierarchy::utility::list_cryptocurrencies::execute(args, db)
}

pub fn handle_rename_wallet_group(args: RenameWalletGroupArgs, db: &Database) -> Result<()> {
    hierarchy::rename::rename_wallet_group::execute(args, db)
}

pub fn handle_rename_address_group(args: RenameAddressGroupArgs, db: &Database) -> Result<()> {
    hierarchy::rename::rename_address_group::execute(args, db)
}

pub fn handle_rename_wallet(args: RenameWalletArgs, db: &Database) -> Result<()> {
    hierarchy::rename::rename_wallet::execute(args, db)
}

pub fn handle_rename_subwallet(args: RenameSubwalletArgs, db: &Database) -> Result<()> {
    hierarchy::rename::rename_subwallet::execute(args, db)
}

pub fn handle_rename_standalone_wallet(args: RenameStandaloneWalletArgs, db: &Database) -> Result<()> {
    hierarchy::rename::rename_standalone_wallet::execute(args, db)
}

pub fn handle_remove_account(args: RemoveAccountArgs, db: &Database) -> Result<()> {
    hierarchy::remove::remove_account::execute(args, db)
}

pub fn handle_remove_wallet_group(args: RemoveWalletGroupArgs, db: &Database) -> Result<()> {
    hierarchy::remove::remove_wallet_group::execute(args, db)
}

pub fn handle_remove_address_group(args: RemoveAddressGroupArgs, db: &Database) -> Result<()> {
    hierarchy::remove::remove_address_group::execute(args, db)
}

pub fn handle_remove_wallet(args: RemoveWalletArgs, db: &Database) -> Result<()> {
    hierarchy::remove::remove_wallet::execute(args, db)
}

pub fn handle_remove_subwallet(args: RemoveSubwalletArgs, db: &Database) -> Result<()> {
    hierarchy::remove::remove_subwallet::execute(args, db)
}

pub fn handle_remove_standalone_wallet(args: RemoveStandaloneWalletArgs, db: &Database) -> Result<()> {
    hierarchy::remove::remove_standalone_wallet::execute(args, db)
}

pub fn handle_modify_wallet(args: ModifyWalletArgs, db: &Database) -> Result<()> {
    hierarchy::wallet::modify_wallet::execute(args, db)
}

pub fn handle_show_wallet(args: ShowWalletArgs, db: &Database) -> Result<()> {
    hierarchy::wallet::show_wallet::execute(args, db)
}

pub fn handle_show_subwallet(args: ShowSubwalletArgs, db: &Database) -> Result<()> {
    hierarchy::subwallet::show_subwallet::execute(args, db)
}

pub fn handle_show_standalone_wallet(args: ShowStandaloneWalletArgs, db: &Database) -> Result<()> {
    hierarchy::standalone::show_standalone_wallet::execute(args, db)
}

pub fn handle_modify_subwallet(args: ModifySubwalletArgs, db: &Database) -> Result<()> {
    hierarchy::subwallet::modify_subwallet::execute(args, db)
}

pub fn handle_modify_standalone_wallet(args: ModifyStandaloneWalletArgs, db: &Database) -> Result<()> {
    hierarchy::standalone::modify_standalone_wallet::execute(args, db)
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