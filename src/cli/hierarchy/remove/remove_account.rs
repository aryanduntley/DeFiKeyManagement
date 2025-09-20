use anyhow::{Result, Context};
use clap::Args;
use crate::database::Database;
use crate::crypto::validate_mnemonic_with_account;

#[derive(Args)]
pub struct RemoveAccountArgs {
    #[arg(long, help = "Name of the master account to remove")]
    pub account: String,
    #[arg(long, help = "Mnemonic phrase for verification (required for account removal)")]
    pub mnemonic: String,
    #[arg(long, help = "Passphrase for mnemonic (if used during account creation)")]
    pub passphrase: Option<String>,
    #[arg(long, help = "Skip confirmation prompt")]
    pub force: bool,
}

pub fn execute(args: RemoveAccountArgs, db: &Database) -> Result<()> {
    println!("🗑️  Removing master account");
    println!("Account: {}", args.account);

    // Get the master account
    let master_account = match db.get_master_account_by_name(&args.account)? {
        Some(account) => account,
        None => {
            println!("\n❌ Master account '{}' not found.", args.account);
            return Ok(());
        }
    };

    // Verify mnemonic matches the account
    let is_valid = validate_mnemonic_with_account(
        &args.mnemonic,
        args.passphrase.as_deref().unwrap_or(""),
        &master_account
    ).context("Failed to validate mnemonic")?;

    if !is_valid {
        println!("\n❌ Invalid mnemonic phrase for account '{}'.", args.account);
        println!("   The provided mnemonic does not match this account.");
        return Ok(());
    }

    // Get wallet groups count for confirmation
    let wallet_groups = db.list_wallet_groups(master_account.id.unwrap())?;
    let wallet_group_count = wallet_groups.len();

    // Warning about cascading deletion
    if !args.force {
        println!("\n⚠️  WARNING: This will permanently delete:");
        println!("   • Master account: {}", args.account);
        println!("   • {} wallet group(s) and all their contents", wallet_group_count);
        println!("   • All wallets, address groups, and subwallets");
        println!("   • All associated private keys and addresses");
        println!("\n❗ This action cannot be undone!");

        print!("\nType 'DELETE' to confirm: ");
        use std::io::{self, Write};
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if input.trim() != "DELETE" {
            println!("❌ Account removal cancelled.");
            return Ok(());
        }
    }

    // Perform cascading deletion
    let success = db.delete_master_account(&args.account, &args.mnemonic)
        .context("Failed to remove master account")?;

    if success {
        println!("\n🎉 Master account removed successfully!");
        println!("   Account: {}", args.account);
        println!("   Removed {} wallet group(s) and all contents", wallet_group_count);

        println!("\n💡 Next steps:");
        println!("   • View remaining accounts: wallet-backup list-accounts");
        println!("   • Create new account: wallet-backup add-account --account \"name\" --mnemonic \"words...\"");
    } else {
        println!("\n❌ Failed to remove master account.");
    }

    Ok(())
}