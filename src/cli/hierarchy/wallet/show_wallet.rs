use anyhow::{Result, Context};
use clap::Args;
use crate::database::Database;

#[derive(Args)]
pub struct ShowWalletArgs {
    #[arg(long, help = "Name of the master account")]
    pub account: String,
    #[arg(long, help = "Name of the wallet group")]
    pub wallet_group: String,
    #[arg(long, help = "Name or address of the wallet to show")]
    pub wallet: String,
    #[arg(long, help = "Include sensitive data (private key)")]
    pub include_sensitive: bool,
}

pub fn execute(args: ShowWalletArgs, db: &Database) -> Result<()> {
    println!("📱 Wallet Details");
    println!("Account: {}", args.account);
    println!("Wallet Group: {}", args.wallet_group);

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

    // Get all wallets in the wallet group to search through
    let wallets = db.get_wallets_by_wallet_group(wallet_group.id.unwrap())
        .context("Failed to get wallets")?;

    // Find the target wallet by name or address
    let target_wallet = wallets.into_iter().find(|w| {
        // Try to match by label first, then by address
        (if let Some(label) = &w.label {
            label == &args.wallet
        } else {
            false
        }) || w.address == args.wallet
    });

    let wallet = match target_wallet {
        Some(w) => w,
        None => {
            println!("❌ Wallet '{}' not found in wallet group '{}'.", args.wallet, args.wallet_group);
            return Ok(());
        }
    };

    // Display wallet information
    println!("\n📱 Wallet Information");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Always show all public information
    println!("   🔗 Blockchain: {}", wallet.blockchain);
    println!("   📍 Address: {}", wallet.address);

    // Show checksum address (if available)
    if let Some(checksum) = &wallet.address_with_checksum {
        if !checksum.is_empty() && checksum != &wallet.address {
            println!("   ✅ Checksum Address: {}", checksum);
        } else {
            println!("   ✅ Checksum Address: (same as address)");
        }
    } else {
        println!("   ✅ Checksum Address: (none)");
    }

    // Show public key (if available)
    if let Some(pub_key) = &wallet.public_key {
        if !pub_key.is_empty() {
            println!("   🔑 Public Key: {}", pub_key);
        } else {
            println!("   🔑 Public Key: (empty)");
        }
    } else {
        println!("   🔑 Public Key: (none)");
    }

    // Show derivation path
    if let Some(derivation) = &wallet.derivation_path {
        if !derivation.is_empty() {
            println!("   🛤️  Derivation Path: {}", derivation);
        } else {
            println!("   🛤️  Derivation Path: (empty)");
        }
    } else {
        println!("   🛤️  Derivation Path: (none)");
    }

    // Always show label field
    if let Some(label) = &wallet.label {
        if !label.is_empty() {
            println!("   🏷️  Label: {}", label);
        } else {
            println!("   🏷️  Label: (empty)");
        }
    } else {
        println!("   🏷️  Label: (none)");
    }

    // Show explorer URL (if available)
    if let Some(explorer) = &wallet.explorer_url {
        if !explorer.is_empty() {
            println!("   🌐 Explorer URL: {}", explorer);
        } else {
            println!("   🌐 Explorer URL: (empty)");
        }
    } else {
        println!("   🌐 Explorer URL: (none)");
    }

    // Always show notes field
    if let Some(notes) = &wallet.notes {
        if !notes.is_empty() {
            println!("   📝 Notes: {}", notes);
        } else {
            println!("   📝 Notes: (empty)");
        }
    } else {
        println!("   📝 Notes: (none)");
    }

    // Show creation date
    println!("   📅 Created At: {}", wallet.created_at);

    // Show additional data
    if wallet.additional_data.is_empty() {
        println!("   📊 Additional Data: (none)");
    } else {
        println!("   📊 Additional Data:");
        for (key, value) in &wallet.additional_data {
            println!("      {} = {}", key, value);
        }
    }

    // Show secondary addresses
    if wallet.secondary_addresses.is_empty() {
        println!("   🏠 Secondary Addresses: (none)");
    } else {
        println!("   🏠 Secondary Addresses:");
        for (addr_type, address) in &wallet.secondary_addresses {
            println!("      {}: {}", addr_type, address);
        }
    }

    // Show private key only if sensitive flag is used
    if args.include_sensitive {
        println!("\n🔒 Sensitive Information");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("   🔑 Private Key: {}", wallet.private_key);
    } else {
        println!("\n🔒 Sensitive Information");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("   🔑 Private Key: (use --include-sensitive to view)");
    }

    // Show address groups count
    let address_groups = db.list_address_groups_for_wallet(wallet.id.unwrap())
        .context("Failed to get address groups")?;

    if address_groups.is_empty() {
        println!("\n📁 Address Groups: (none)");
    } else {
        println!("\n📁 Address Groups ({}):", address_groups.len());
        for group in &address_groups {
            println!("   • {}", group.name);
        }
    }

    println!("\n💡 Next steps:");
    println!("   • View address groups: wallet-backup list-address-groups --account \"{}\" --wallet-group \"{}\" --wallet \"{}\"", args.account, args.wallet_group, args.wallet);
    if !address_groups.is_empty() {
        println!("   • View specific address group: wallet-backup show-address-group --account \"{}\" --wallet-group \"{}\" --wallet \"{}\" --address-group \"<group_name>\"", args.account, args.wallet_group, args.wallet);
    }
    println!("   • Modify wallet: wallet-backup modify-wallet --account \"{}\" --wallet-group \"{}\" --wallet \"{}\"", args.account, args.wallet_group, args.wallet);

    Ok(())
}