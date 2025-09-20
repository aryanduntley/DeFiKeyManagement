use anyhow::{Result, Context};
use clap::Args;
use crate::database::Database;
use crate::crypto::validate_mnemonic_with_account;

#[derive(Args)]
pub struct RemoveWalletGroupArgs {
    #[arg(long, help = "Name of the master account")]
    pub account: String,
    #[arg(long, help = "Name of the wallet group to remove")]
    pub wallet_group: String,
    #[arg(long, help = "Mnemonic phrase for verification (required for wallet group removal)")]
    pub mnemonic: String,
    #[arg(long, help = "Passphrase for mnemonic (if used during account creation)")]
    pub passphrase: Option<String>,
    #[arg(long, help = "Skip confirmation prompt")]
    pub force: bool,
}

pub fn execute(args: RemoveWalletGroupArgs, db: &Database) -> Result<()> {
    println!("🗑️  Removing wallet group");
    println!("Account: {}", args.account);
    println!("Wallet Group: {}", args.wallet_group);

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

    // Get the wallet group
    let wallet_group = match db.get_wallet_group_by_name(master_account.id.unwrap(), &args.wallet_group)? {
        Some(group) => group,
        None => {
            println!("❌ Wallet group '{}' not found.", args.wallet_group);
            return Ok(());
        }
    };

    // Check if wallet group has any wallets - prevent removal if not empty
    let wallets = db.get_wallets_by_wallet_group(wallet_group.id.unwrap())?;
    let wallet_count = wallets.len();

    if wallet_count > 0 {
        println!("\n❌ Cannot remove wallet group '{}' - it contains {} wallet(s).", args.wallet_group, wallet_count);
        println!("   For security, only empty wallet groups can be removed.");
        println!("   Please remove all wallets first:");
        for wallet in &wallets {
            if let Some(label) = &wallet.label {
                println!("     • wallet-backup remove-wallet --account \"{}\" --wallet-group \"{}\" --wallet-name \"{}\" --mnemonic \"<mnemonic>\"",
                    args.account, args.wallet_group, label);
            } else {
                println!("     • wallet-backup remove-wallet --account \"{}\" --wallet-group \"{}\" --address \"{}\" --mnemonic \"<mnemonic>\"",
                    args.account, args.wallet_group, wallet.address);
            }
        }
        return Ok(());
    }

    // Confirmation for empty wallet group removal
    if !args.force {
        println!("\n⚠️  WARNING: This will permanently delete:");
        println!("   • Wallet group: {}", args.wallet_group);
        println!("   • The wallet group is empty (no wallets)");
        println!("\n❗ This action cannot be undone!");

        print!("\nType 'DELETE' to confirm: ");
        use std::io::{self, Write};
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if input.trim() != "DELETE" {
            println!("❌ Wallet group removal cancelled.");
            return Ok(());
        }
    }

    // Perform deletion
    let success = db.delete_wallet_group(master_account.id.unwrap(), &args.wallet_group)
        .context("Failed to remove wallet group")?;

    if success {
        println!("\n🎉 Wallet group removed successfully!");
        println!("   Account: {}", args.account);
        println!("   Wallet Group: {}", args.wallet_group);
        println!("   Wallet group was empty (0 wallets)");

        println!("\n💡 Next steps:");
        println!("   • View remaining wallet groups: wallet-backup list-wallet-groups --account \"{}\"", args.account);
        println!("   • Create new wallet group: wallet-backup add-wallet-group --account \"{}\" --name \"group_name\"", args.account);
    } else {
        println!("\n❌ Failed to remove wallet group.");
    }

    Ok(())
}