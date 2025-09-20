use anyhow::{Result, Context};
use clap::Args;
use crate::database::Database;

#[derive(Args)]
pub struct RenameSubwalletArgs {
    #[arg(long, help = "Name of the master account")]
    pub account: String,
    #[arg(long, help = "Name of the wallet group")]
    pub wallet_group: String,
    #[arg(long, help = "Name of the base wallet")]
    pub wallet: String,
    #[arg(long, help = "Name of the address group")]
    pub address_group: String,
    #[arg(long, help = "Current name of the subwallet", conflicts_with = "address")]
    pub old_name: Option<String>,
    #[arg(long, help = "Address of the subwallet", conflicts_with = "old_name")]
    pub address: Option<String>,
    #[arg(long, help = "New name for the subwallet")]
    pub new_name: String,
}

pub fn execute(args: RenameSubwalletArgs, db: &Database) -> Result<()> {
    println!("✏️  Renaming subwallet");
    println!("Account: {}", args.account);
    println!("Wallet Group: {}", args.wallet_group);
    println!("Base Wallet: {}", args.wallet);
    println!("Address Group: {}", args.address_group);

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

    // Navigate to address group
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
    let target_subwallet = if let Some(_) = args.old_name {
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
            if args.old_name.is_some() {
                println!("❌ Subwallet with name '{}' not found in address group '{}'.", identifier, args.address_group);
            } else {
                println!("❌ Subwallet with address '{}' not found in address group '{}'.", identifier, args.address_group);
            }
            return Ok(());
        }
    };

    // Perform the rename using the subwallet's address
    let success = db.update_wallet_label(&subwallet.address, &args.new_name)
        .context("Failed to rename subwallet")?;

    if success {
        println!("\n🎉 Subwallet renamed successfully!");
        if let Some(old_label) = &subwallet.label {
            println!("   {} → {}", old_label, args.new_name);
        } else {
            println!("   (unnamed) → {}", args.new_name);
        }
        println!("   Address: {}", subwallet.address);

        println!("\n💡 Next steps:");
        println!("   • View subwallet: wallet-backup show-wallet --address \"{}\"", subwallet.address);
        println!("   • List subwallets: wallet-backup list-subwallets --account \"{}\" --wallet-group \"{}\" --wallet \"{}\" --address-group \"{}\"", args.account, args.wallet_group, args.wallet, args.address_group);
    } else {
        println!("\n❌ Failed to rename subwallet.");
    }

    Ok(())
}