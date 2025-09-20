use anyhow::{Result, Context};
use clap::Args;
use crate::database::Database;

#[derive(Args)]
pub struct RenameWalletArgs {
    #[arg(long, help = "Name of the master account")]
    pub account: String,
    #[arg(long, help = "Name of the wallet group")]
    pub wallet_group: String,
    #[arg(long, help = "Current name of the wallet", conflicts_with = "address")]
    pub old_name: Option<String>,
    #[arg(long, help = "Address of the wallet", conflicts_with = "old_name")]
    pub address: Option<String>,
    #[arg(long, help = "New name for the wallet")]
    pub new_name: String,
}

pub fn execute(args: RenameWalletArgs, db: &Database) -> Result<()> {
    println!("✏️  Renaming wallet");
    println!("Account: {}", args.account);
    println!("Wallet Group: {}", args.wallet_group);

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

    // Navigate to wallet group
    let master_account = match db.get_master_account_by_name(&args.account)? {
        Some(account) => account,
        None => {
            println!("\n❌ Master account '{}' not found.", args.account);
            return Ok(());
        }
    };

    let wallet_group = match db.get_wallet_group_by_name(master_account.id.unwrap(), &args.wallet_group)? {
        Some(group) => group,
        None => {
            println!("❌ Wallet group '{}' not found.", args.wallet_group);
            return Ok(());
        }
    };

    // Find the wallet by name or address
    let target_wallet = if let Some(_) = args.old_name {
        // Find by name
        db.get_wallet_by_name_in_group(wallet_group.id.unwrap(), &identifier)?
    } else {
        // Find by address
        db.get_wallet_by_address(&identifier)?
    };

    let wallet = match target_wallet {
        Some(w) => w,
        None => {
            if args.old_name.is_some() {
                println!("❌ Wallet with name '{}' not found in wallet group '{}'.", identifier, args.wallet_group);
            } else {
                println!("❌ Wallet with address '{}' not found.", identifier);
            }
            return Ok(());
        }
    };

    // Perform the rename using the wallet's address
    let success = db.update_wallet_label(&wallet.address, &args.new_name)
        .context("Failed to rename wallet")?;

    if success {
        println!("\n🎉 Wallet renamed successfully!");
        if let Some(old_label) = &wallet.label {
            println!("   {} → {}", old_label, args.new_name);
        } else {
            println!("   (unnamed) → {}", args.new_name);
        }
        println!("   Address: {}", wallet.address);

        println!("\n💡 Next steps:");
        println!("   • View wallet: wallet-backup show-wallet --address \"{}\"", wallet.address);
        println!("   • List wallets: wallet-backup list-wallets --account \"{}\" --wallet-group \"{}\"", args.account, args.wallet_group);
    } else {
        println!("\n❌ Failed to rename wallet.");
    }

    Ok(())
}