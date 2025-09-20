use clap::{Parser, Subcommand};
use anyhow::Result;

mod cli;
mod crypto;
mod database;
mod blockchain;
mod utils;

use cli::*;
use database::Database;

#[derive(Parser)]
#[command(name = "wallet-backup")]
#[command(about = "Multi-chain cryptocurrency wallet backup and key management tool")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new account (hierarchical wallet root)
    AddAccount(CreateMasterArgs),
    /// List all accounts
    ListAccounts(ListAccountsArgs),
    /// Show detailed information for a master account
    ShowAccount(ShowAccountArgs),
    /// Create a new wallet group under an account
    AddWalletGroup(CreateWalletGroupArgs),
    /// Add a single wallet to a wallet group for a specific blockchain
    AddWallet(AddWalletArgs),
    /// List wallet groups for an account
    ListWalletGroups(ListWalletGroupsArgs),
    /// Show detailed information for a wallet group
    ShowWalletGroup(ShowWalletGroupArgs),
    /// Show detailed information for an address group
    ShowAddressGroup(ShowAddressGroupArgs),
    /// Add a standalone wallet from a private key
    AddStandaloneWallet(AddStandaloneWalletArgs),
    /// Add an address group to a wallet group for organizing wallets
    AddAddressGroup(AddAddressGroupArgs),
    /// List wallets within a wallet group
    ListWallets(ListWalletsArgs),
    /// List all standalone wallets (imported from private keys)
    ListStandaloneWallets(ListStandaloneWalletsArgs),
    /// List address groups within a wallet
    ListAddressGroups(ListAddressGroupsArgs),
    /// Add a subwallet (grandchild private key) to an address group
    AddSubwallet(AddSubwalletArgs),
    /// List subwallets within an address group
    ListSubwallets(ListSubwalletsArgs),
    /// List all supported cryptocurrencies/blockchains
    ListCryptocurrencies(ListCryptocurrenciesArgs),
    /// Rename a wallet group
    RenameWalletGroup(RenameWalletGroupArgs),
    /// Rename an address group
    RenameAddressGroup(RenameAddressGroupArgs),
    /// Rename a wallet
    RenameWallet(RenameWalletArgs),
    /// Rename a subwallet
    RenameSubwallet(RenameSubwalletArgs),
    /// Rename a standalone wallet
    RenameStandaloneWallet(RenameStandaloneWalletArgs),
    /// Remove an account with all associated data
    RemoveAccount(RemoveAccountArgs),
    /// Remove a wallet group with all associated wallets
    RemoveWalletGroup(RemoveWalletGroupArgs),
    /// Remove an address group with all associated subwallets
    RemoveAddressGroup(RemoveAddressGroupArgs),
    /// Remove a wallet with all associated address groups
    RemoveWallet(RemoveWalletArgs),
    /// Remove a subwallet (grandchild private key)
    RemoveSubwallet(RemoveSubwalletArgs),
    /// Remove a standalone wallet
    RemoveStandaloneWallet(RemoveStandaloneWalletArgs),
    /// Modify wallet properties (label, notes, additional data, secondary addresses)
    ModifyWallet(ModifyWalletArgs),
    /// Show detailed information for a specific wallet
    ShowWallet(ShowWalletArgs),
    /// Show detailed information for a specific subwallet
    ShowSubwallet(ShowSubwalletArgs),
    /// Show detailed information for a standalone wallet
    ShowStandaloneWallet(ShowStandaloneWalletArgs),
    /// Modify subwallet properties (label, notes, additional data, secondary addresses)
    ModifySubwallet(ModifySubwalletArgs),
    /// Modify standalone wallet properties (label, notes, additional data, secondary addresses)
    ModifyStandaloneWallet(ModifyStandaloneWalletArgs),
    // TEMPORARILY DISABLED - TO BE REPLACED
    // /// Import a wallet from mnemonic or private key
    // Import(ImportArgs),
    // /// Import multiple wallets from one mnemonic across selected blockchains
    // ImportMulti(ImportMultiArgs),
    // /// Derive keys/addresses from mnemonic
    // Derive(DeriveArgs),
    // /// List all stored wallets
    // List,
    // /// Show detailed wallet information
    // Show(ShowArgs),
    // /// Get wallet by name (alias for show --label)
    // Get {
    //     /// Wallet name/label to retrieve
    //     name: String,
    //     /// Include sensitive data (private key, mnemonic)
    //     #[arg(long)]
    //     include_sensitive: bool,
    // },
    // /// Export wallet data
    // Export(ExportArgs),
    // /// Delete a wallet
    // Delete(DeleteArgs),
    // /// Update wallet label
    // Tag(TagArgs),
    // /// Search wallets
    // Search(SearchArgs),
    // /// List wallet groups
    // ListGroups,
    // /// Show detailed information for a wallet group
    // ShowGroup(ShowGroupArgs),
    // /// Add blockchains to an existing wallet group
    // DeriveMulti(DeriveMultiArgs),
    // /// Rename a wallet group
    // RenameGroup(RenameGroupArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize database
    let db = Database::new("wallets.db")?;
    
    // Execute command
    match cli.command {
        Commands::AddAccount(args) => handle_create_master(args, &db),
        Commands::ListAccounts(args) => handle_list_accounts(args, &db),
        Commands::ShowAccount(args) => handle_show_account(args, &db),
        Commands::AddWalletGroup(args) => handle_create_wallet_group(args, &db),
        Commands::AddWallet(args) => handle_add_wallet(args, &db),
        Commands::ListWalletGroups(args) => handle_list_wallet_groups(args, &db),
        Commands::ShowWalletGroup(args) => handle_show_wallet_group(args, &db),
        Commands::ShowAddressGroup(args) => handle_show_address_group(args, &db),
        Commands::AddStandaloneWallet(args) => handle_add_standalone_wallet(args, &db),
        Commands::AddAddressGroup(args) => handle_add_address_group(args, &db),
        Commands::ListWallets(args) => handle_list_wallets(args, &db),
        Commands::ListStandaloneWallets(args) => handle_list_standalone_wallets(args, &db),
        Commands::ListAddressGroups(args) => handle_list_address_groups(args, &db),
        Commands::AddSubwallet(args) => handle_add_subwallet(args, &db),
        Commands::ListSubwallets(args) => handle_list_subwallets(args, &db),
        Commands::ListCryptocurrencies(args) => handle_list_cryptocurrencies(args, &db),
        Commands::RenameWalletGroup(args) => handle_rename_wallet_group(args, &db),
        Commands::RenameAddressGroup(args) => handle_rename_address_group(args, &db),
        Commands::RenameWallet(args) => handle_rename_wallet(args, &db),
        Commands::RenameSubwallet(args) => handle_rename_subwallet(args, &db),
        Commands::RenameStandaloneWallet(args) => handle_rename_standalone_wallet(args, &db),
        Commands::RemoveAccount(args) => handle_remove_account(args, &db),
        Commands::RemoveWalletGroup(args) => handle_remove_wallet_group(args, &db),
        Commands::RemoveAddressGroup(args) => handle_remove_address_group(args, &db),
        Commands::RemoveWallet(args) => handle_remove_wallet(args, &db),
        Commands::RemoveSubwallet(args) => handle_remove_subwallet(args, &db),
        Commands::RemoveStandaloneWallet(args) => handle_remove_standalone_wallet(args, &db),
        Commands::ModifyWallet(args) => handle_modify_wallet(args, &db),
        Commands::ShowWallet(args) => handle_show_wallet(args, &db),
        Commands::ShowSubwallet(args) => handle_show_subwallet(args, &db),
        Commands::ShowStandaloneWallet(args) => handle_show_standalone_wallet(args, &db),
        Commands::ModifySubwallet(args) => handle_modify_subwallet(args, &db),
        Commands::ModifyStandaloneWallet(args) => handle_modify_standalone_wallet(args, &db),
        // TEMPORARILY DISABLED
        // Commands::Import(args) => handle_import(args, &db),
        // Commands::ImportMulti(args) => handle_import_multi(args, &db),
        // Commands::Derive(args) => handle_derive(args, &db),
        // Commands::List => handle_list(&db),
        // Commands::Show(args) => handle_show(args, &db),
        // Commands::Get { name, include_sensitive } => {
        //     let args = ShowArgs {
        //         address: None,
        //         label: Some(name),
        //         include_sensitive,
        //     };
        //     handle_show(args, &db)
        // },
        // Commands::Export(args) => handle_export(args, &db),
        // Commands::Delete(args) => handle_delete(args, &db),
        // Commands::Tag(args) => handle_tag(args, &db),
        // Commands::Search(args) => handle_search(args, &db),
        // Commands::ListGroups => handle_list_groups(&db),
        // Commands::ShowGroup(args) => handle_show_group(args, &db),
        // Commands::DeriveMulti(args) => handle_derive_multi(args, &db),
        // Commands::RenameGroup(args) => handle_rename_group(args, &db),
    }
}
