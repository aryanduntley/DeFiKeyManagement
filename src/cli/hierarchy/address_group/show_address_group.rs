use anyhow::{Result, Context};
use clap::Args;
use crate::database::Database;

#[derive(Args)]
pub struct ShowAddressGroupArgs {
    #[arg(long, help = "Name of the master account")]
    pub account: String,
    #[arg(long, help = "Name of the wallet group")]
    pub wallet_group: String,
    #[arg(long, help = "Name or address of the wallet")]
    pub wallet: String,
    #[arg(long, help = "Name of the address group to show")]
    pub group_name: String,
    #[arg(long, help = "Include sensitive information (private keys)")]
    pub include_sensitive: bool,
}

pub fn execute(args: ShowAddressGroupArgs, db: &Database) -> Result<()> {
    println!("🗂️  Address Group Details");
    println!("Account: {}", args.account);
    println!("Wallet Group: {}", args.wallet_group);
    println!("Base Wallet: {}", args.wallet);
    println!("Address Group: {}", args.group_name);

    // Get master account by name
    let master_account = match db.get_master_account_by_name(&args.account)? {
        Some(account) => account,
        None => {
            println!("\n❌ Master account '{}' not found.", args.account);
            return Ok(());
        }
    };

    // Get wallet group by name
    let wallet_group = match db.get_wallet_group_by_name(master_account.id.unwrap(), &args.wallet_group)? {
        Some(group) => group,
        None => {
            println!("❌ Wallet group '{}' not found in account '{}'.", args.wallet_group, args.account);
            return Ok(());
        }
    };

    // Get all wallets in the wallet group to find the base wallet
    let wallets = db.get_wallets_by_wallet_group(wallet_group.id.unwrap())
        .context("Failed to get wallets")?;

    // Find the base wallet by name or address
    let base_wallet = wallets.into_iter().find(|w| {
        (if let Some(label) = &w.label {
            label == &args.wallet
        } else {
            false
        }) || w.address == args.wallet
    });

    let wallet = match base_wallet {
        Some(w) => w,
        None => {
            println!("❌ Base wallet '{}' not found in wallet group '{}'.", args.wallet, args.wallet_group);
            return Ok(());
        }
    };

    // Get address group by name within the wallet
    let address_group = match db.get_address_group_by_name_for_wallet(wallet.id.unwrap(), &args.group_name)? {
        Some(group) => group,
        None => {
            println!("❌ Address group '{}' not found in wallet '{}'.", args.group_name, args.wallet);
            return Ok(());
        }
    };

    // Get all subwallets in the address group
    let subwallets = db.get_wallets_by_address_group(address_group.id.unwrap())
        .context("Failed to get subwallets")?;

    // Display address group information
    println!("\n🗂️  Address Group Information");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("   📛 Group Name: {}", address_group.name);
    println!("   🔗 Blockchain: {}", wallet.blockchain);
    println!("   📅 Created At: {}", address_group.created_at);

    if subwallets.is_empty() {
        println!("\n📱 Subwallets: (none)");
        println!("   Add a subwallet: wallet-backup add-subwallet --account \"{}\" --wallet-group \"{}\" --wallet \"{}\" --address-group \"{}\" --name \"addr1\"", args.account, args.wallet_group, args.wallet, args.group_name);
    } else {
        println!("\n📱 Subwallets ({}):", subwallets.len());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        for (i, subwallet) in subwallets.iter().enumerate() {
            println!("   {}. 📱 {}", i + 1,
                subwallet.label.as_ref().unwrap_or(&format!("(unnamed)")));
            println!("      📍 Address: {}", subwallet.address);

            // Show checksum address (if available)
            if let Some(checksum) = &subwallet.address_with_checksum {
                if !checksum.is_empty() && checksum != &subwallet.address {
                    println!("      ✅ Checksum: {}", checksum);
                }
            }

            // Show derivation path
            if let Some(derivation) = &subwallet.derivation_path {
                if !derivation.is_empty() {
                    println!("      🛤️  Path: {}", derivation);
                }
            }

            // Show label
            if let Some(label) = &subwallet.label {
                if !label.is_empty() {
                    println!("      🏷️  Label: {}", label);
                }
            }

            // Show notes if present
            if let Some(notes) = &subwallet.notes {
                if !notes.is_empty() {
                    println!("      📝 Notes: {}", notes);
                }
            }

            // Show additional data if present
            if !subwallet.additional_data.is_empty() {
                println!("      📊 Additional Data:");
                for (key, value) in &subwallet.additional_data {
                    println!("         {} = {}", key, value);
                }
            }

            // Show secondary addresses if present
            if !subwallet.secondary_addresses.is_empty() {
                println!("      🏠 Secondary Addresses:");
                for (addr_type, address) in &subwallet.secondary_addresses {
                    println!("         {}: {}", addr_type, address);
                }
            }

            // Show private key only if sensitive flag is used
            if args.include_sensitive {
                println!("      🔒 Private Key: {}", subwallet.private_key);
            } else {
                println!("      🔒 Private Key: (use --include-sensitive to view)");
            }

            if i < subwallets.len() - 1 {
                println!();
            }
        }
    }

    println!("\n💡 Next steps:");
    println!("   • List all subwallets: wallet-backup list-subwallets --account \"{}\" --wallet-group \"{}\" --wallet \"{}\" --address-group \"{}\"", args.account, args.wallet_group, args.wallet, args.group_name);
    if !subwallets.is_empty() {
        println!("   • Show specific subwallet: wallet-backup show-subwallet --account \"{}\" --wallet-group \"{}\" --wallet \"{}\" --address-group \"{}\" --subwallet \"<subwallet-name>\"", args.account, args.wallet_group, args.wallet, args.group_name);
    }
    println!("   • View parent wallet: wallet-backup show-wallet --account \"{}\" --wallet-group \"{}\" --wallet \"{}\"", args.account, args.wallet_group, args.wallet);

    Ok(())
}