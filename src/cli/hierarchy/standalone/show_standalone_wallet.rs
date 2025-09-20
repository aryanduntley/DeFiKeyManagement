use anyhow::{Result, Context};
use clap::Args;
use crate::database::Database;

#[derive(Args)]
pub struct ShowStandaloneWalletArgs {
    #[arg(long, help = "Name of the standalone wallet to show", conflicts_with = "address")]
    pub name: Option<String>,
    #[arg(long, help = "Address of the standalone wallet to show", conflicts_with = "name")]
    pub address: Option<String>,
    #[arg(long, help = "Include sensitive data (private key)")]
    pub include_sensitive: bool,
}

pub fn execute(args: ShowStandaloneWalletArgs, db: &Database) -> Result<()> {
    println!("📱 Standalone Wallet Details");

    // Validate that either name or address is provided
    let identifier = if let Some(ref name) = args.name {
        println!("Wallet Name: {}", name);
        name.clone()
    } else if let Some(ref addr) = args.address {
        println!("Wallet Address: {}", addr);
        addr.clone()
    } else {
        println!("❌ Either --name or --address must be provided.");
        return Ok(());
    };

    // Get all standalone wallets
    let standalone_wallets = db.get_standalone_wallets()
        .context("Failed to get standalone wallets")?;

    // Find the target wallet by name or address
    let target_wallet = if args.name.is_some() {
        // Find by name
        standalone_wallets.into_iter().find(|w| {
            w.label.as_ref().map_or(false, |label| label == &identifier)
        })
    } else {
        // Find by address
        standalone_wallets.into_iter().find(|w| w.address == identifier)
    };

    let wallet = match target_wallet {
        Some(w) => w,
        None => {
            if args.name.is_some() {
                println!("❌ Standalone wallet with name '{}' not found.", identifier);
            } else {
                println!("❌ Standalone wallet with address '{}' not found.", identifier);
            }
            return Ok(());
        }
    };

    // Display wallet information
    println!("\n📱 Standalone Wallet Information");
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

    // Show derivation path (if available - standalone wallets may not have one)
    if let Some(derivation) = &wallet.derivation_path {
        if !derivation.is_empty() {
            println!("   🛤️  Derivation Path: {}", derivation);
        } else {
            println!("   🛤️  Derivation Path: (empty)");
        }
    } else {
        println!("   🛤️  Derivation Path: (none - imported from private key)");
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

    println!("\n💡 Next steps:");
    println!("   • View all standalone wallets: wallet-backup list-standalone-wallets");
    if let Some(label) = &wallet.label {
        println!("   • Modify wallet: wallet-backup modify-standalone-wallet --name \"{}\"", label);
    } else {
        println!("   • Modify wallet: wallet-backup modify-standalone-wallet --address \"{}\"", wallet.address);
    }

    Ok(())
}