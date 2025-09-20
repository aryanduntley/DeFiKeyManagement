use anyhow::{Result, Context};
use clap::Args;
use crate::database::Database;

#[derive(Args)]
pub struct RenameWalletGroupArgs {
    #[arg(long, help = "Name of the master account")]
    pub account: String,
    #[arg(long, help = "Current name of the wallet group")]
    pub old_name: String,
    #[arg(long, help = "New name for the wallet group")]
    pub new_name: String,
}

pub fn execute(args: RenameWalletGroupArgs, db: &Database) -> Result<()> {
    println!("✏️  Renaming wallet group");
    println!("Account: {}", args.account);
    println!("Old Name: {}", args.old_name);
    println!("New Name: {}", args.new_name);

    // Get master account by name
    let master_account = match db.get_master_account_by_name(&args.account)? {
        Some(account) => account,
        None => {
            println!("\n❌ Master account '{}' not found.", args.account);
            return Ok(());
        }
    };

    // Check if old wallet group exists
    let old_group = match db.get_wallet_group_by_name(master_account.id.unwrap(), &args.old_name)? {
        Some(group) => group,
        None => {
            println!("❌ Wallet group '{}' not found in account '{}'.", args.old_name, args.account);
            println!("   Use 'wallet-backup list-wallet-groups --account \"{}\"' to see available groups.", args.account);
            return Ok(());
        }
    };

    // Check if new name already exists
    if let Some(_) = db.get_wallet_group_by_name(master_account.id.unwrap(), &args.new_name)? {
        println!("❌ Wallet group '{}' already exists in account '{}'.", args.new_name, args.account);
        println!("   Choose a different name for the rename operation.");
        return Ok(());
    }

    // Perform the rename
    let success = db.rename_wallet_group(master_account.id.unwrap(), &args.old_name, &args.new_name)
        .context("Failed to rename wallet group")?;

    if success {
        println!("\n🎉 Wallet group renamed successfully!");
        println!("   {} → {}", args.old_name, args.new_name);
        println!("   Account: {}", args.account);
        println!("   Account Index: {}", old_group.account_index);

        println!("\n💡 Next steps:");
        println!("   • View renamed group: wallet-backup show-wallet-group --account \"{}\" --wallet-group \"{}\"", args.account, args.new_name);
        println!("   • List all groups: wallet-backup list-wallet-groups --account \"{}\"", args.account);
    } else {
        println!("\n❌ Failed to rename wallet group. The operation did not complete successfully.");
    }

    Ok(())
}