use anyhow::{Result, Context};
use clap::Args;
use crate::database::Database;

#[derive(Args)]
pub struct RenameAddressGroupArgs {
    #[arg(long, help = "Name of the master account")]
    pub account: String,
    #[arg(long, help = "Name of the wallet group")]
    pub wallet_group: String,
    #[arg(long, help = "Name of the base wallet")]
    pub wallet: String,
    #[arg(long, help = "Current name of the address group")]
    pub old_name: String,
    #[arg(long, help = "New name for the address group")]
    pub new_name: String,
}

pub fn execute(args: RenameAddressGroupArgs, db: &Database) -> Result<()> {
    println!("✏️  Renaming address group");
    println!("Account: {}", args.account);
    println!("Wallet Group: {}", args.wallet_group);
    println!("Base Wallet: {}", args.wallet);
    println!("Old Name: {}", args.old_name);
    println!("New Name: {}", args.new_name);

    // Navigate to base wallet
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

    let base_wallet = match db.get_wallet_by_name_in_group(wallet_group.id.unwrap(), &args.wallet)? {
        Some(wallet) => wallet,
        None => {
            println!("❌ Wallet '{}' not found.", args.wallet);
            return Ok(());
        }
    };

    // Check if old address group exists
    if let None = db.get_address_group_by_name_for_wallet(base_wallet.id.unwrap(), &args.old_name)? {
        println!("❌ Address group '{}' not found for wallet '{}'.", args.old_name, args.wallet);
        return Ok(());
    }

    // Check if new name already exists
    if let Some(_) = db.get_address_group_by_name_for_wallet(base_wallet.id.unwrap(), &args.new_name)? {
        println!("❌ Address group '{}' already exists for wallet '{}'.", args.new_name, args.wallet);
        return Ok(());
    }

    // Perform the rename
    let success = db.rename_address_group(base_wallet.id.unwrap(), &args.old_name, &args.new_name)
        .context("Failed to rename address group")?;

    if success {
        println!("\n🎉 Address group renamed successfully!");
        println!("   {} → {}", args.old_name, args.new_name);
        println!("   Base Wallet: {}", args.wallet);

        println!("\n💡 Next steps:");
        println!("   • View subwallets: wallet-backup list-subwallets --account \"{}\" --wallet-group \"{}\" --wallet \"{}\" --address-group \"{}\"", args.account, args.wallet_group, args.wallet, args.new_name);
    } else {
        println!("\n❌ Failed to rename address group.");
    }

    Ok(())
}