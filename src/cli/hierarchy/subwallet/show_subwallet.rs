use anyhow::{Result, Context};
use clap::Args;
use crate::database::Database;

#[derive(Args)]
pub struct ShowSubwalletArgs {
    #[arg(long, help = "Name of the master account")]
    pub account: String,
    #[arg(long, help = "Name of the wallet group")]
    pub wallet_group: String,
    #[arg(long, help = "Name or address of the wallet")]
    pub wallet: String,
    #[arg(long, help = "Name of the address group")]
    pub address_group: String,
    #[arg(long, help = "Name or address of the subwallet to show")]
    pub subwallet: String,
    #[arg(long, help = "Include sensitive data (private key)")]
    pub include_sensitive: bool,
}

pub fn execute(args: ShowSubwalletArgs, db: &Database) -> Result<()> {
    println!("📱 Subwallet Details");
    println!("Account: {}", args.account);
    println!("Wallet Group: {}", args.wallet_group);
    println!("Wallet: {}", args.wallet);
    println!("Address Group: {}", args.address_group);

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
            println!("❌ Wallet '{}' not found in wallet group '{}'.", args.wallet, args.wallet_group);
            return Ok(());
        }
    };

    // Get address group by name within the wallet
    let address_group = match db.get_address_group_by_name_for_wallet(wallet.id.unwrap(), &args.address_group)? {
        Some(group) => group,
        None => {
            println!("❌ Address group '{}' not found in wallet '{}'.", args.address_group, args.wallet);
            return Ok(());
        }
    };

    // Get all subwallets in the address group
    let subwallets = db.get_wallets_by_address_group(address_group.id.unwrap())
        .context("Failed to get subwallets")?;

    // Find the target subwallet by name or address
    let target_subwallet = subwallets.into_iter().find(|s| {
        (if let Some(label) = &s.label {
            label == &args.subwallet
        } else {
            false
        }) || s.address == args.subwallet
    });

    let subwallet = match target_subwallet {
        Some(s) => s,
        None => {
            println!("❌ Subwallet '{}' not found in address group '{}'.", args.subwallet, args.address_group);
            return Ok(());
        }
    };

    // Display subwallet information
    println!("\n📱 Subwallet Information");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Always show all public information
    println!("   🔗 Blockchain: {}", subwallet.blockchain);
    println!("   📍 Address: {}", subwallet.address);

    // Show checksum address (if available)
    if let Some(checksum) = &subwallet.address_with_checksum {
        if !checksum.is_empty() && checksum != &subwallet.address {
            println!("   ✅ Checksum Address: {}", checksum);
        } else {
            println!("   ✅ Checksum Address: (same as address)");
        }
    } else {
        println!("   ✅ Checksum Address: (none)");
    }

    // Show public key (if available)
    if let Some(pub_key) = &subwallet.public_key {
        if !pub_key.is_empty() {
            println!("   🔑 Public Key: {}", pub_key);
        } else {
            println!("   🔑 Public Key: (empty)");
        }
    } else {
        println!("   🔑 Public Key: (none)");
    }

    // Show derivation path
    if let Some(derivation) = &subwallet.derivation_path {
        if !derivation.is_empty() {
            println!("   🛤️  Derivation Path: {}", derivation);
        } else {
            println!("   🛤️  Derivation Path: (empty)");
        }
    } else {
        println!("   🛤️  Derivation Path: (none)");
    }

    // Always show label field
    if let Some(label) = &subwallet.label {
        if !label.is_empty() {
            println!("   🏷️  Label: {}", label);
        } else {
            println!("   🏷️  Label: (empty)");
        }
    } else {
        println!("   🏷️  Label: (none)");
    }

    // Show explorer URL (if available)
    if let Some(explorer) = &subwallet.explorer_url {
        if !explorer.is_empty() {
            println!("   🌐 Explorer URL: {}", explorer);
        } else {
            println!("   🌐 Explorer URL: (empty)");
        }
    } else {
        println!("   🌐 Explorer URL: (none)");
    }

    // Always show notes field
    if let Some(notes) = &subwallet.notes {
        if !notes.is_empty() {
            println!("   📝 Notes: {}", notes);
        } else {
            println!("   📝 Notes: (empty)");
        }
    } else {
        println!("   📝 Notes: (none)");
    }

    // Show creation date
    println!("   📅 Created At: {}", subwallet.created_at);

    // Show additional data
    if subwallet.additional_data.is_empty() {
        println!("   📊 Additional Data: (none)");
    } else {
        println!("   📊 Additional Data:");
        for (key, value) in &subwallet.additional_data {
            println!("      {} = {}", key, value);
        }
    }

    // Show secondary addresses
    if subwallet.secondary_addresses.is_empty() {
        println!("   🏠 Secondary Addresses: (none)");
    } else {
        println!("   🏠 Secondary Addresses:");
        for (addr_type, address) in &subwallet.secondary_addresses {
            println!("      {}: {}", addr_type, address);
        }
    }

    // Show private key only if sensitive flag is used
    if args.include_sensitive {
        println!("\n🔒 Sensitive Information");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("   🔑 Private Key: {}", subwallet.private_key);
    } else {
        println!("\n🔒 Sensitive Information");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("   🔑 Private Key: (use --include-sensitive to view)");
    }

    println!("\n💡 Next steps:");
    println!("   • View all subwallets: wallet-backup list-subwallets --account \"{}\" --wallet-group \"{}\" --wallet \"{}\" --address-group \"{}\"", args.account, args.wallet_group, args.wallet, args.address_group);
    println!("   • Modify subwallet: wallet-backup modify-subwallet --account \"{}\" --wallet-group \"{}\" --wallet \"{}\" --address-group \"{}\" --subwallet \"{}\"", args.account, args.wallet_group, args.wallet, args.address_group, args.subwallet);
    println!("   • View parent address group: wallet-backup show-address-group --account \"{}\" --wallet-group \"{}\" --wallet \"{}\" --address-group \"{}\"", args.account, args.wallet_group, args.wallet, args.address_group);

    Ok(())
}