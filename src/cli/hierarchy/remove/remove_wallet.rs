use anyhow::{Result, Context};
use clap::Args;
use crate::database::Database;
use crate::crypto::validate_mnemonic_with_account;

#[derive(Args)]
pub struct RemoveWalletArgs {
    #[arg(long, help = "Name of the master account")]
    pub account: String,
    #[arg(long, help = "Name of the wallet group")]
    pub wallet_group: String,
    #[arg(long, help = "Name of the wallet to remove", conflicts_with = "address")]
    pub wallet_name: Option<String>,
    #[arg(long, help = "Address of the wallet to remove", conflicts_with = "wallet_name")]
    pub address: Option<String>,
    #[arg(long, help = "Mnemonic phrase for verification (required for wallet removal)")]
    pub mnemonic: String,
    #[arg(long, help = "Passphrase for mnemonic (if used during account creation)")]
    pub passphrase: Option<String>,
    #[arg(long, help = "Skip confirmation prompt")]
    pub force: bool,
}

pub fn execute(args: RemoveWalletArgs, db: &Database) -> Result<()> {
    println!("🗑️  Removing wallet");
    println!("Account: {}", args.account);
    println!("Wallet Group: {}", args.wallet_group);

    let identifier = if let Some(ref name) = args.wallet_name {
        println!("Wallet Name: {}", name);
        name.clone()
    } else if let Some(ref addr) = args.address {
        println!("Wallet Address: {}", addr);
        addr.clone()
    } else {
        println!("❌ Either --wallet-name or --address must be provided.");
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

    // Navigate to the wallet
    let wallet_group = match db.get_wallet_group_by_name(master_account.id.unwrap(), &args.wallet_group)? {
        Some(group) => group,
        None => {
            println!("❌ Wallet group '{}' not found.", args.wallet_group);
            return Ok(());
        }
    };

    // Get all wallets in the wallet group to search through
    let wallets = db.get_wallets_by_wallet_group(wallet_group.id.unwrap())
        .context("Failed to get wallets")?;

    // Find the target wallet by name or address
    let target_wallet = if let Some(_) = args.wallet_name {
        // Find by name
        wallets.into_iter().find(|w| {
            w.label.as_ref().map_or(false, |label| label == &identifier)
        })
    } else {
        // Find by address
        wallets.into_iter().find(|w| w.address == identifier)
    };

    let wallet = match target_wallet {
        Some(w) => w,
        None => {
            if args.wallet_name.is_some() {
                println!("❌ Wallet with name '{}' not found in wallet group '{}'.", identifier, args.wallet_group);
            } else {
                println!("❌ Wallet with address '{}' not found in wallet group '{}'.", identifier, args.wallet_group);
            }
            return Ok(());
        }
    };

    // Get address groups count for confirmation
    let address_groups = db.list_address_groups_for_wallet(wallet.id.unwrap())?;
    let address_group_count = address_groups.len();

    let mut total_subwallets = 0;
    for address_group in &address_groups {
        let subwallets = db.get_wallets_by_address_group(address_group.id)?;
        total_subwallets += subwallets.len();
    }

    // Warning about cascading deletion
    if !args.force {
        println!("\n⚠️  WARNING: This will permanently delete:");
        println!("   • Wallet: {}", wallet.label.as_ref().unwrap_or(&wallet.address));
        println!("   • {} address group(s)", address_group_count);
        println!("   • {} subwallet(s) and all their contents", total_subwallets);
        println!("   • All associated private keys and addresses");
        println!("\n❗ This action cannot be undone!");

        print!("\nType 'DELETE' to confirm: ");
        use std::io::{self, Write};
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if input.trim() != "DELETE" {
            println!("❌ Wallet removal cancelled.");
            return Ok(());
        }
    }

    // Perform deletion using the wallet's address
    let success = db.delete_wallet(&wallet.address, Some(&args.mnemonic))
        .context("Failed to remove wallet")?;

    if success {
        println!("\n🎉 Wallet removed successfully!");
        println!("   Account: {}", args.account);
        println!("   Wallet Group: {}", args.wallet_group);
        if let Some(label) = &wallet.label {
            println!("   Wallet: {}", label);
        }
        println!("   Address: {}", wallet.address);
        println!("   Removed {} address group(s) and {} subwallet(s)", address_group_count, total_subwallets);

        println!("\n💡 Next steps:");
        println!("   • View remaining wallets: wallet-backup list-wallets --account \"{}\" --wallet-group \"{}\"", args.account, args.wallet_group);
        println!("   • Create new wallet: wallet-backup add-wallet --account \"{}\" --wallet-group \"{}\" --blockchain \"blockchain_name\" --name \"wallet_name\"", args.account, args.wallet_group);
    } else {
        println!("\n❌ Failed to remove wallet.");
    }

    Ok(())
}