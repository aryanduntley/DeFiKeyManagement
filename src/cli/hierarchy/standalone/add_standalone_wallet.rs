use anyhow::{Result, Context};
use clap::Args;
use chrono::Utc;

use crate::database::{Database, Wallet};
use crate::blockchain::{SupportedBlockchain, get_blockchain_handler};

#[derive(Args)]
pub struct AddStandaloneWalletArgs {
    #[arg(long, help = "Private key for the wallet (hex format)")]
    pub private_key: String,
    #[arg(long, help = "Blockchain for the wallet (e.g., 'bitcoin', 'ethereum', 'solana')")]
    pub blockchain: String,
    #[arg(long, help = "Name/label for the wallet")]
    pub name: String,
    #[arg(long, help = "Optional notes for the wallet")]
    pub notes: Option<String>,
}

pub fn execute(args: AddStandaloneWalletArgs, db: &Database) -> Result<()> {
    println!("Adding standalone wallet: {}", args.name);
    println!("Blockchain: {}", args.blockchain);

    // Parse and validate blockchain
    let blockchain = match SupportedBlockchain::from_str(&args.blockchain.to_lowercase()) {
        Ok(chain) => chain,
        Err(_) => {
            println!("❌ Invalid blockchain: {}", args.blockchain);
            println!("   Supported blockchains: {}", SupportedBlockchain::get_supported_blockchain_names().join(", "));
            return Ok(());
        }
    };

    println!("✓ Blockchain validated: {}", blockchain);

    // Check if wallet with this name already exists
    if let Some(_existing) = db.get_wallet_by_label(&args.name)? {
        println!("❌ Wallet with name '{}' already exists.", args.name);
        println!("   Use a different name or update the existing wallet.");
        return Ok(());
    }

    // Get blockchain handler
    let handler = get_blockchain_handler(&blockchain)?;

    // Derive wallet keys from private key
    let wallet_keys = handler.derive_from_private_key(&args.private_key)
        .context("Failed to derive wallet from private key")?;

    println!("✓ Wallet keys derived from private key");
    println!("  Address: {}", wallet_keys.address);

    // Create Wallet record for standalone wallet
    let wallet = Wallet {
        id: None,
        wallet_group_id: None, // NULL for standalone wallet
        address_group_id: None, // NULL for standalone wallet
        blockchain: blockchain.to_string(),
        address: wallet_keys.address.clone(),
        address_with_checksum: wallet_keys.address_with_checksum.clone(),
        private_key: wallet_keys.private_key,
        public_key: Some(wallet_keys.public_key),
        derivation_path: None, // No derivation path for imported private key
        label: Some(args.name.clone()),
        source_type: "private_key".to_string(),
        explorer_url: Some(blockchain.get_explorer_url(&wallet_keys.address)),
        notes: args.notes.clone(),
        created_at: Utc::now(),
        additional_data: wallet_keys.additional_data,
        secondary_addresses: wallet_keys.secondary_addresses,
    };

    // Insert into database
    let wallet_id = db.create_wallet(&wallet)?;

    // Success message
    println!("\n🎉 Standalone wallet created successfully!");
    println!("   Wallet Name: {}", args.name);
    println!("   Wallet ID: {}", wallet_id);
    println!("   Blockchain: {}", blockchain);
    println!("   Address: {}", wallet_keys.address);

    if let Some(checksum) = &wallet_keys.address_with_checksum {
        if checksum != &wallet_keys.address {
            println!("   Checksum Address: {}", checksum);
        }
    }

    if let Some(explorer) = &wallet.explorer_url {
        println!("   Explorer: {}", explorer);
    }

    if let Some(notes) = &args.notes {
        println!("   Notes: {}", notes);
    }

    println!("\n💡 Next steps:");
    println!("   1. List all standalone wallets: wallet-backup list-standalone-wallets");
    println!("   2. Show wallet details: wallet-backup show-wallet --name \"{}\"", args.name);

    Ok(())
}