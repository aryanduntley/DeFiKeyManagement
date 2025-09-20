use anyhow::{Result, Context};
use clap::Args;
use crate::database::Database;
use crate::crypto::validate_mnemonic_with_account;

#[derive(Args)]
pub struct RemoveAddressGroupArgs {
    #[arg(long, help = "Name of the master account")]
    pub account: String,
    #[arg(long, help = "Name of the wallet group")]
    pub wallet_group: String,
    #[arg(long, help = "Name of the base wallet")]
    pub wallet: String,
    #[arg(long, help = "Name of the address group to remove")]
    pub address_group: String,
    #[arg(long, help = "Mnemonic phrase for verification (required for address group removal)")]
    pub mnemonic: String,
    #[arg(long, help = "Passphrase for mnemonic (if used during account creation)")]
    pub passphrase: Option<String>,
    #[arg(long, help = "Skip confirmation prompt")]
    pub force: bool,
}

pub fn execute(args: RemoveAddressGroupArgs, db: &Database) -> Result<()> {
    println!("🗑️  Removing address group");
    println!("Account: {}", args.account);
    println!("Wallet Group: {}", args.wallet_group);
    println!("Base Wallet: {}", args.wallet);
    println!("Address Group: {}", args.address_group);

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

    // Navigate to the address group
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
            println!("❌ Base wallet '{}' not found.", args.wallet);
            return Ok(());
        }
    };

    let address_group = match db.get_address_group_by_name_for_wallet(base_wallet.id.unwrap(), &args.address_group)? {
        Some(group) => group,
        None => {
            println!("❌ Address group '{}' not found.", args.address_group);
            return Ok(());
        }
    };

    // Check if address group has any subwallets - prevent removal if not empty
    let subwallets = db.get_wallets_by_address_group(address_group.id.unwrap())?;
    let subwallet_count = subwallets.len();

    if subwallet_count > 0 {
        println!("\n❌ Cannot remove address group '{}' - it contains {} subwallet(s).", args.address_group, subwallet_count);
        println!("   For security, only empty address groups can be removed.");
        println!("   Please remove all subwallets first:");
        for subwallet in &subwallets {
            if let Some(label) = &subwallet.label {
                println!("     • wallet-backup remove-subwallet --account \"{}\" --wallet-group \"{}\" --wallet \"{}\" --address-group \"{}\" --subwallet-name \"{}\" --mnemonic \"<mnemonic>\"",
                    args.account, args.wallet_group, args.wallet, args.address_group, label);
            } else {
                println!("     • wallet-backup remove-subwallet --account \"{}\" --wallet-group \"{}\" --wallet \"{}\" --address-group \"{}\" --address \"{}\" --mnemonic \"<mnemonic>\"",
                    args.account, args.wallet_group, args.wallet, args.address_group, subwallet.address);
            }
        }
        return Ok(());
    }

    // Confirmation for empty address group removal
    if !args.force {
        println!("\n⚠️  WARNING: This will permanently delete:");
        println!("   • Address group: {}", args.address_group);
        println!("   • The address group is empty (no subwallets)");
        println!("\n❗ This action cannot be undone!");

        print!("\nType 'DELETE' to confirm: ");
        use std::io::{self, Write};
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if input.trim() != "DELETE" {
            println!("❌ Address group removal cancelled.");
            return Ok(());
        }
    }

    // Perform deletion
    let success = db.delete_address_group(base_wallet.id.unwrap(), &args.address_group, &args.mnemonic)
        .context("Failed to remove address group")?;

    if success {
        println!("\n🎉 Address group removed successfully!");
        println!("   Account: {}", args.account);
        println!("   Wallet Group: {}", args.wallet_group);
        println!("   Base Wallet: {}", args.wallet);
        println!("   Address Group: {}", args.address_group);
        println!("   Address group was empty (0 subwallets)");

        println!("\n💡 Next steps:");
        println!("   • View remaining address groups: wallet-backup list-address-groups --account \"{}\" --wallet-group \"{}\" --wallet \"{}\"", args.account, args.wallet_group, args.wallet);
        println!("   • Create new address group: wallet-backup add-address-group --account \"{}\" --wallet-group \"{}\" --wallet \"{}\" --name \"group_name\"", args.account, args.wallet_group, args.wallet);
    } else {
        println!("\n❌ Failed to remove address group.");
    }

    Ok(())
}