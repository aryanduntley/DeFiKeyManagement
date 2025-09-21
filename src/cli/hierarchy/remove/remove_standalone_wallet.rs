use anyhow::{Result, Context};
use clap::Args;
use crate::database::Database;

#[derive(Args)]
pub struct RemoveStandaloneWalletArgs {
    #[arg(long, help = "Name of the standalone wallet to remove", conflicts_with = "address")]
    pub wallet: Option<String>,
    #[arg(long, help = "Address of the standalone wallet to remove", conflicts_with = "wallet")]
    pub address: Option<String>,
    #[arg(long, help = "Private key for verification (required for standalone wallet removal)")]
    pub private_key: String,
    #[arg(long, help = "Skip confirmation prompt")]
    pub force: bool,
}

pub fn execute(args: RemoveStandaloneWalletArgs, db: &Database) -> Result<()> {
    println!("🗑️  Removing standalone wallet");

    let identifier = if let Some(ref name) = args.wallet {
        println!("Wallet Name: {}", name);
        name.clone()
    } else if let Some(ref addr) = args.address {
        println!("Wallet Address: {}", addr);
        addr.clone()
    } else {
        println!("❌ Either --wallet or --address must be provided.");
        return Ok(());
    };

    // Get all standalone wallets to search through
    let standalone_wallets = db.get_standalone_wallets()
        .context("Failed to get standalone wallets")?;

    // Find the target standalone wallet by name or address
    let target_wallet = if let Some(_) = args.wallet {
        // Find by name
        standalone_wallets.into_iter().find(|w| {
            w.label.as_ref().map_or(false, |label| label == &identifier)
        })
    } else {
        // Find by address
        standalone_wallets.into_iter().find(|w| w.address == identifier)
    };

    let wallet = match target_wallet {
        Some(w) => w,
        None => {
            if args.wallet.is_some() {
                println!("❌ Standalone wallet with name '{}' not found.", identifier);
            } else {
                println!("❌ Standalone wallet with address '{}' not found.", identifier);
            }
            return Ok(());
        }
    };

    // Verify private key matches the wallet
    if wallet.private_key != args.private_key {
        println!("\n❌ Invalid private key for standalone wallet.");
        println!("   The provided private key does not match this wallet.");
        return Ok(());
    }

    // Warning about deletion
    if !args.force {
        println!("\n⚠️  WARNING: This will permanently delete:");
        println!("   • Standalone wallet: {}", wallet.label.as_ref().unwrap_or(&wallet.address));
        println!("   • Address: {}", wallet.address);
        println!("   • Private key and all associated data");
        println!("\n❗ This action cannot be undone!");

        print!("\nType 'DELETE' to confirm: ");
        use std::io::{self, Write};
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if input.trim() != "DELETE" {
            println!("❌ Standalone wallet removal cancelled.");
            return Ok(());
        }
    }

    // Perform deletion using the wallet's address (no mnemonic needed for standalone)
    let success = db.delete_wallet(&wallet.address, None)
        .context("Failed to remove standalone wallet")?;

    if success {
        println!("\n🎉 Standalone wallet removed successfully!");
        if let Some(label) = &wallet.label {
            println!("   Wallet: {}", label);
        }
        println!("   Address: {}", wallet.address);
        println!("   Blockchain: {}", wallet.blockchain);

        println!("\n💡 Next steps:");
        println!("   • View remaining standalone wallets: wallet-backup list-standalone-wallets");
        println!("   • Import new standalone wallet: wallet-backup add-standalone-wallet --private-key \"key\" --blockchain \"blockchain_name\" --name \"wallet_name\"");
    } else {
        println!("\n❌ Failed to remove standalone wallet.");
    }

    Ok(())
}