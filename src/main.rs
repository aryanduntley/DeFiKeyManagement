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
    /// Import a wallet from mnemonic or private key
    Import(ImportArgs),
    /// Import multiple wallets from one mnemonic across selected blockchains
    ImportMulti(ImportMultiArgs),
    /// Derive keys/addresses from mnemonic
    Derive(DeriveArgs),
    /// List all stored wallets
    List,
    /// Show detailed wallet information
    Show(ShowArgs),
    /// Get wallet by name (alias for show --label)
    Get {
        /// Wallet name/label to retrieve
        name: String,
        /// Include sensitive data (private key, mnemonic)
        #[arg(long)]
        include_sensitive: bool,
    },
    /// Export wallet data
    Export(ExportArgs),
    /// Delete a wallet
    Delete(DeleteArgs),
    /// Update wallet label
    Tag(TagArgs),
    /// Search wallets
    Search(SearchArgs),
    /// List wallet groups
    ListGroups,
    /// Show detailed information for a wallet group
    ShowGroup(ShowGroupArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize database
    let db = Database::new("wallets.db")?;
    
    // Execute command
    match cli.command {
        Commands::Import(args) => handle_import(args, &db),
        Commands::ImportMulti(args) => handle_import_multi(args, &db),
        Commands::Derive(args) => handle_derive(args, &db),
        Commands::List => handle_list(&db),
        Commands::Show(args) => handle_show(args, &db),
        Commands::Get { name, include_sensitive } => {
            let args = ShowArgs {
                address: None,
                label: Some(name),
                include_sensitive,
            };
            handle_show(args, &db)
        },
        Commands::Export(args) => handle_export(args, &db),
        Commands::Delete(args) => handle_delete(args, &db),
        Commands::Tag(args) => handle_tag(args, &db),
        Commands::Search(args) => handle_search(args, &db),
        Commands::ListGroups => handle_list_groups(&db),
        Commands::ShowGroup(args) => handle_show_group(args, &db),
    }
}
