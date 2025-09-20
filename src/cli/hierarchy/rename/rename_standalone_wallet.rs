use anyhow::{Result, Context};
use clap::Args;
use crate::database::Database;

#[derive(Args)]
pub struct RenameStandaloneWalletArgs {
    #[arg(long, help = "Current name of the standalone wallet", conflicts_with = "address")]
    pub old_name: Option<String>,
    #[arg(long, help = "Address of the standalone wallet", conflicts_with = "old_name")]
    pub address: Option<String>,
    #[arg(long, help = "New name for the standalone wallet")]
    pub new_name: String,
}

pub fn execute(args: RenameStandaloneWalletArgs, db: &Database) -> Result<()> {
    println!("✏️  Renaming standalone wallet");

    let identifier = if let Some(ref name) = args.old_name {
        println!("Old Name: {}", name);
        name.clone()
    } else if let Some(ref addr) = args.address {
        println!("Address: {}", addr);
        addr.clone()
    } else {
        println!("❌ Either --old-name or --address must be provided.");
        return Ok(());
    };

    println!("New Name: {}", args.new_name);

    // Get all standalone wallets and find the target wallet
    let standalone_wallets = db.get_standalone_wallets()
        .context("Failed to get standalone wallets")?;

    let target_wallet = if args.old_name.is_some() {
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
            if args.old_name.is_some() {
                println!("❌ Standalone wallet with name '{}' not found.", identifier);
            } else {
                println!("❌ Standalone wallet with address '{}' not found.", identifier);
            }
            return Ok(());
        }
    };

    // Perform the rename using the wallet's address
    let success = db.update_wallet_label(&wallet.address, &args.new_name)
        .context("Failed to rename standalone wallet")?;

    if success {
        println!("\n🎉 Standalone wallet renamed successfully!");
        if let Some(old_label) = &wallet.label {
            println!("   {} → {}", old_label, args.new_name);
        } else {
            println!("   (unnamed) → {}", args.new_name);
        }
        println!("   Address: {}", wallet.address);

        println!("\n💡 Next steps:");
        println!("   • View wallet: wallet-backup show-standalone-wallet --name \"{}\"", args.new_name);
        println!("   • List standalone wallets: wallet-backup list-standalone-wallets");
    } else {
        println!("\n❌ Failed to rename standalone wallet.");
    }

    Ok(())
}