use anyhow::{Result, Context};
use crate::cli::ShowGroupArgs;
use crate::database::Database;

pub fn execute(args: ShowGroupArgs, db: &Database) -> Result<()> {
    println!("Wallet Group Details:");
    println!("{:-<80}", "");

    // Get wallet group by name
    let group = db.get_wallet_group_by_name(&args.group_name)
        .context("Failed to retrieve wallet group")?;

    let group = match group {
        Some(g) => g,
        None => {
            println!("❌ Wallet group '{}' not found.", args.group_name);
            println!("\nAvailable groups:");

            // Show available groups
            let groups = db.get_all_wallet_groups().context("Failed to list groups")?;
            if groups.is_empty() {
                println!("  (no groups found)");
                println!("\nUse 'wallet-backup import-multi' to create a wallet group.");
            } else {
                for g in groups {
                    println!("  - {}", g.name);
                }
                println!("\nUse 'wallet-backup list-groups' to see detailed group information.");
            }
            return Ok(());
        }
    };

    // Display group information
    println!("📁 Group: {}", group.name);
    if let Some(desc) = &group.description {
        println!("📝 Description: {}", desc);
    }
    println!("🔗 Blockchains: {}", group.blockchains.join(", "));
    println!("📅 Created: {}", group.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
    println!();

    // Get wallets in this group
    let group_id = group.id.expect("Group should have ID");
    let wallets = db.get_wallets_by_group_id(group_id)
        .context("Failed to retrieve wallets for group")?;

    if wallets.is_empty() {
        println!("⚠️  No wallets found in this group.");
        println!("This may indicate a database consistency issue.");
        return Ok(());
    }

    println!("💰 Wallets ({} total):", wallets.len());
    println!("{:-<80}", "");

    // Group wallets by blockchain for organized display
    let mut blockchain_groups: std::collections::HashMap<String, Vec<_>> = std::collections::HashMap::new();
    for wallet in &wallets {
        blockchain_groups.entry(wallet.blockchain.clone())
            .or_insert_with(Vec::new)
            .push(wallet);
    }

    // Display wallets organized by blockchain
    for blockchain in &group.blockchains {
        if let Some(blockchain_wallets) = blockchain_groups.get(blockchain) {
            println!("\n🔸 {} ({} wallet{})",
                     blockchain.to_uppercase(),
                     blockchain_wallets.len(),
                     if blockchain_wallets.len() == 1 { "" } else { "s" });

            for wallet in blockchain_wallets {
                print_wallet_info(wallet, args.include_sensitive);
            }
        }
    }

    // Show any wallets from blockchains not in the group (edge case)
    for (blockchain, blockchain_wallets) in &blockchain_groups {
        if !group.blockchains.contains(blockchain) {
            println!("\n⚠️  {} (not in group blockchain list - may be orphaned)", blockchain.to_uppercase());
            for wallet in blockchain_wallets {
                print_wallet_info(wallet, args.include_sensitive);
            }
        }
    }

    println!("\n{:-<80}", "");
    println!("📊 Summary: {} wallet{} across {} blockchain{}",
             wallets.len(),
             if wallets.len() == 1 { "" } else { "s" },
             group.blockchains.len(),
             if group.blockchains.len() == 1 { "" } else { "s" });

    if !args.include_sensitive {
        println!("\n💡 Use --include-sensitive to show private keys and mnemonics");
    }

    Ok(())
}

fn print_wallet_info(wallet: &crate::database::WalletRecord, include_sensitive: bool) {
    println!("  ├─ 📍 Address: {}", wallet.address);

    // Show address with checksum if different
    if let Some(checksum_addr) = &wallet.address_with_checksum {
        if checksum_addr != &wallet.address {
            println!("  │  ✓ Checksum: {}", checksum_addr);
        }
    }

    println!("  ├─ 🛤️  Path: {}", wallet.derivation_path);

    if let Some(account) = wallet.account {
        if let Some(index) = wallet.address_index {
            println!("  ├─ 🔢 Account: {}, Index: {}", account, index);
        }
    }

    // Show secondary addresses if any
    if !wallet.secondary_addresses.is_empty() {
        println!("  ├─ 🔄 Secondary addresses:");
        for (addr_type, addr) in &wallet.secondary_addresses {
            println!("  │  └─ {}: {}", addr_type, addr);
        }
    }

    // Show additional data if any
    if !wallet.additional_data.is_empty() {
        println!("  ├─ 📋 Additional data:");
        for (key, value) in &wallet.additional_data {
            println!("  │  └─ {}: {}", key, value);
        }
    }

    if let Some(explorer_url) = &wallet.explorer_url {
        println!("  ├─ 🔍 Explorer: {}", explorer_url);
    }

    if include_sensitive {
        println!("  ├─ 🔑 Private Key: {}", wallet.private_key);
        if let Some(mnemonic) = &wallet.mnemonic {
            println!("  ├─ 🌱 Mnemonic: {}", mnemonic);
        }
        if let Some(passphrase) = &wallet.passphrase {
            println!("  ├─ 🔐 Passphrase: {}", passphrase);
        }
    }

    if let Some(label) = &wallet.label {
        println!("  └─ 🏷️  Label: {}", label);
    } else {
        println!("  └─");
    }
    println!();
}