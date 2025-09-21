use anyhow::{Result, Context};
use clap::Args;
use crate::database::Database;
use crate::crypto::validate_mnemonic_with_account;

#[derive(Args)]
pub struct RemoveSubwalletArgs {
    #[arg(long, help = "Name of the master account")]
    pub account: String,
    #[arg(long, help = "Name of the wallet group")]
    pub wallet_group: String,
    #[arg(long, help = "Name of the base wallet")]
    pub wallet: String,
    #[arg(long, help = "Name of the address group")]
    pub address_group: String,
    #[arg(long, help = "Name of the subwallet to remove", conflicts_with = "address")]
    pub subwallet: Option<String>,
    #[arg(long, help = "Address of the subwallet to remove", conflicts_with = "subwallet")]
    pub address: Option<String>,
    #[arg(long, help = "Mnemonic phrase for verification (required for subwallet removal)")]
    pub mnemonic: String,
    #[arg(long, help = "Passphrase for mnemonic (if used during account creation)")]
    pub passphrase: Option<String>,
    #[arg(long, help = "Skip confirmation prompt")]
    pub force: bool,
}

pub fn execute(args: RemoveSubwalletArgs, db: &Database) -> Result<()> {
    println!("🗑️  Removing subwallet");
    println!("Account: {}", args.account);
    println!("Wallet Group: {}", args.wallet_group);
    println!("Base Wallet: {}", args.wallet);
    println!("Address Group: {}", args.address_group);

    let identifier = if let Some(ref name) = args.subwallet {
        println!("Subwallet Name: {}", name);
        name.clone()
    } else if let Some(ref addr) = args.address {
        println!("Subwallet Address: {}", addr);
        addr.clone()
    } else {
        println!("❌ Either --subwallet or --address must be provided.");
        return Ok(());
    };

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

    // Navigate to the subwallet
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

    // Get all subwallets in the address group to search through
    let subwallets = db.get_wallets_by_address_group(address_group.id.unwrap())
        .context("Failed to get subwallets")?;

    // Find the target subwallet by name or address
    let target_subwallet = if let Some(_) = args.subwallet {
        // Find by name
        subwallets.into_iter().find(|w| {
            w.label.as_ref().map_or(false, |label| label == &identifier)
        })
    } else {
        // Find by address
        subwallets.into_iter().find(|w| w.address == identifier)
    };

    let subwallet = match target_subwallet {
        Some(w) => w,
        None => {
            if args.subwallet.is_some() {
                println!("❌ Subwallet with name '{}' not found in address group '{}'.", identifier, args.address_group);
            } else {
                println!("❌ Subwallet with address '{}' not found in address group '{}'.", identifier, args.address_group);
            }
            return Ok(());
        }
    };

    // Warning about deletion
    if !args.force {
        println!("\n⚠️  WARNING: This will permanently delete:");
        println!("   • Subwallet: {}", subwallet.label.as_ref().unwrap_or(&subwallet.address));
        println!("   • Address: {}", subwallet.address);
        println!("   • Grandchild private key and all associated data");
        println!("\n❗ This action cannot be undone!");

        print!("\nType 'DELETE' to confirm: ");
        use std::io::{self, Write};
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if input.trim() != "DELETE" {
            println!("❌ Subwallet removal cancelled.");
            return Ok(());
        }
    }

    // Perform deletion using the subwallet's address
    let success = db.delete_wallet(&subwallet.address, Some(&args.mnemonic))
        .context("Failed to remove subwallet")?;

    if success {
        println!("\n🎉 Subwallet removed successfully!");
        println!("   Account: {}", args.account);
        println!("   Wallet Group: {}", args.wallet_group);
        println!("   Base Wallet: {}", args.wallet);
        println!("   Address Group: {}", args.address_group);
        if let Some(label) = &subwallet.label {
            println!("   Subwallet: {}", label);
        }
        println!("   Address: {}", subwallet.address);

        println!("\n💡 Next steps:");
        println!("   • View remaining subwallets: wallet-backup list-subwallets --account \"{}\" --wallet-group \"{}\" --wallet \"{}\" --address-group \"{}\"", args.account, args.wallet_group, args.wallet, args.address_group);
        println!("   • Create new subwallet: wallet-backup add-subwallet --account \"{}\" --wallet-group \"{}\" --wallet \"{}\" --address-group \"{}\" --name \"subwallet_name\"", args.account, args.wallet_group, args.wallet, args.address_group);
    } else {
        println!("\n❌ Failed to remove subwallet.");
    }

    Ok(())
}