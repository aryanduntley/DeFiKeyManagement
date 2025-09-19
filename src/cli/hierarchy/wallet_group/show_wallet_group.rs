use anyhow::{Result, Context};
use clap::Args;
use crate::database::Database;

#[derive(Args)]
pub struct ShowWalletGroupArgs {
    #[arg(long, help = "Name of the master account")]
    pub account: String,
    #[arg(long, help = "Name of the wallet group to show")]
    pub group: String,
    #[arg(long, help = "Include sensitive information (private keys)")]
    pub include_sensitive: bool,
}

pub fn execute(args: ShowWalletGroupArgs, db: &Database) -> Result<()> {
    println!("🔍 Wallet Group Details");
    println!("Master Account: {}", args.account);
    println!("Group Name: {}", args.group);

    // Get master account by name
    let master_account = match db.get_master_account_by_name(&args.account)? {
        Some(account) => account,
        None => {
            println!("\n❌ Master account '{}' not found.", args.account);
            return Ok(());
        }
    };

    // Get wallet group by name
    let wallet_group = match db.get_wallet_group_by_name(master_account.id.unwrap(), &args.group)? {
        Some(group) => group,
        None => {
            println!("\n❌ Wallet group '{}' not found in account '{}'.", args.group, args.account);
            println!("   Use 'wallet-backup list-wallet-groups --account \"{}\"' to see available groups.", args.account);
            return Ok(());
        }
    };

    // Display wallet group summary
    println!("\n📊 Group Summary:");
    println!("   Group ID: {}", wallet_group.id.unwrap_or(-1));
    println!("   Group Name: {}", wallet_group.name);
    println!("   Account Index: {} (BIP-44 account level)", wallet_group.account_index);
    println!("   Created: {}", wallet_group.created_at.format("%Y-%m-%d %H:%M:%S UTC"));

    if let Some(desc) = &wallet_group.description {
        println!("   Description: {}", desc);
    }

    // Get address groups for this wallet group
    let address_groups = db.list_address_groups(wallet_group.id.unwrap(), None)
        .context("Failed to list address groups")?;

    if address_groups.is_empty() {
        println!("\n📝 No blockchains added to this group yet.");
        println!("   Add blockchains: wallet-backup add-blockchain --account \"{}\" --wallet-group \"{}\" --blockchains \"bitcoin,ethereum\"", args.account, args.group);
        return Ok(());
    }

    println!("\n🔗 Blockchains ({} total):", address_groups.len());
    println!("   {:<15} {:<20} {:<12} {:<15} {:<12}",
             "Blockchain", "Address Group", "Group Index", "Address Count", "Created");
    println!("   {}", "─".repeat(80));

    for addr_group in &address_groups {
        println!("   {:<15} {:<20} {:<12} {:<15} {:<12}",
                 addr_group.blockchain,
                 truncate_string(&addr_group.name, 18),
                 addr_group.address_group_index,
                 addr_group.address_count,
                 addr_group.created_at.format("%Y-%m-%d").to_string()
        );
    }

    // Get wallet addresses for this group
    println!("\n💰 Addresses:");

    let mut total_addresses = 0;
    for addr_group in &address_groups {
        let addresses = db.get_wallet_addresses_by_address_group(addr_group.id)
            .context("Failed to get addresses for address group")?;

        if addresses.is_empty() {
            continue;
        }

        println!("\n   📋 {} ({} address{}):",
                 addr_group.blockchain,
                 addresses.len(),
                 if addresses.len() == 1 { "" } else { "es" }
        );

        for addr in &addresses {
            total_addresses += 1;

            println!("      Address: {}", addr.address);

            if let Some(checksum) = &addr.address_with_checksum {
                if checksum != &addr.address {
                    println!("      Checksum: {}", checksum);
                }
            }

            if let Some(derivation_path) = &addr.derivation_path {
                println!("      Derivation: {}", derivation_path);
            }

            if let Some(label) = &addr.label {
                println!("      Label: {}", label);
            }

            if let Some(explorer) = &addr.explorer_url {
                println!("      Explorer: {}", explorer);
            }

            if args.include_sensitive {
                println!("      Private Key: {}", addr.private_key);
                if let Some(public_key) = &addr.public_key {
                    println!("      Public Key: {}", public_key);
                }
            }

            // Show additional data if present
            if !addr.additional_data.is_empty() {
                println!("      Additional Data:");
                for (key, value) in &addr.additional_data {
                    println!("        {}: {}", key, value);
                }
            }

            // Show secondary addresses if present
            if !addr.secondary_addresses.is_empty() {
                println!("      Secondary Addresses:");
                for (addr_type, address) in &addr.secondary_addresses {
                    println!("        {} format: {}", addr_type, address);
                }
            }

            println!("      Created: {}", addr.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
            println!();
        }
    }

    println!("📈 Summary:");
    println!("   Total Blockchains: {}", address_groups.len());
    println!("   Total Addresses: {}", total_addresses);
    println!("   Derivation Account Index: {}", wallet_group.account_index);

    if args.include_sensitive {
        println!("\n⚠️  Sensitive information displayed. Keep this data secure!");
    } else {
        println!("\n💡 To view private keys, use: --include-sensitive");
    }

    println!("\n💡 Next steps:");
    println!("   • Add more blockchains: wallet-backup add-blockchain --account \"{}\" --wallet-group \"{}\" --blockchains \"<blockchain-list>\"", args.account, args.group);
    println!("   • Generate more addresses: wallet-backup generate-address --account \"{}\" --wallet-group \"{}\" --blockchain \"<blockchain>\"", args.account, args.group);

    Ok(())
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}