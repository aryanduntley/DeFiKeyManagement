use anyhow::{Result, Context};
use clap::Args;
use crate::database::Database;

#[derive(Args)]
pub struct ListSubwalletsArgs {
    #[arg(long, help = "Name of the master account")]
    pub account: String,
    #[arg(long, help = "Name of the wallet group")]
    pub wallet_group: String,
    #[arg(long, help = "Name of the base wallet")]
    pub wallet: String,
    #[arg(long, help = "Name of the address group")]
    pub address_group: String,
}

pub fn execute(args: ListSubwalletsArgs, db: &Database) -> Result<()> {
    println!("📋 Listing subwallets in address group");
    println!("Account: {}", args.account);
    println!("Wallet Group: {}", args.wallet_group);
    println!("Base Wallet: {}", args.wallet);
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
            println!("\n❌ Wallet group '{}' not found in account '{}'.", args.wallet_group, args.account);
            println!("   Use 'wallet-backup list-wallet-groups --account \"{}\"' to see available groups.", args.account);
            return Ok(());
        }
    };

    // Get the base wallet by name within this wallet group
    let base_wallet = match db.get_wallet_by_name_in_group(wallet_group.id.unwrap(), &args.wallet)? {
        Some(wallet) => wallet,
        None => {
            println!("\n❌ Base wallet '{}' not found in wallet group '{}'.", args.wallet, args.wallet_group);
            println!("   Use 'wallet-backup list-wallets --account \"{}\" --wallet-group \"{}\"' to see available wallets.", args.account, args.wallet_group);
            return Ok(());
        }
    };

    // Get the address group by name for this wallet
    let address_group = match db.get_address_group_by_name_for_wallet(base_wallet.id.unwrap(), &args.address_group)? {
        Some(group) => group,
        None => {
            println!("\n❌ Address group '{}' not found for wallet '{}'.", args.address_group, args.wallet);
            println!("   Use 'wallet-backup list-address-groups --account \"{}\" --wallet-group \"{}\" --wallet \"{}\"' to see available address groups.", args.account, args.wallet_group, args.wallet);
            return Ok(());
        }
    };

    // Get subwallets in this address group (address_group_id = Some(id))
    // These are the grandchild private keys that belong to this address group
    let subwallets = db.get_wallets_by_address_group(address_group.id.unwrap())
        .context("Failed to get subwallets for address group")?;

    if subwallets.is_empty() {
        println!("\n📝 No subwallets found in this address group.");
        println!("   Add a subwallet: wallet-backup add-subwallet --account \"{}\" --wallet-group \"{}\" --wallet \"{}\" --address-group \"{}\" --name \"MySubwallet\"", args.account, args.wallet_group, args.wallet, args.address_group);
        return Ok(());
    }

    println!("\n💰 Subwallets ({} total):", subwallets.len());
    println!("   {:<20} {:<15} {:<45} {:<25} {:<12}",
             "Subwallet Name", "Blockchain", "Address", "Derivation Path", "Created");
    println!("   {}", "─".repeat(125));

    for subwallet in &subwallets {
        let subwallet_name = subwallet.label.as_deref().unwrap_or("(unnamed)");
        let derivation_path = subwallet.derivation_path.as_deref().unwrap_or("N/A");
        let created_date = subwallet.created_at.format("%Y-%m-%d").to_string();

        println!("   {:<20} {:<15} {:<45} {:<25} {:<12}",
                 truncate_string(subwallet_name, 18),
                 subwallet.blockchain,
                 truncate_string(&subwallet.address, 43),
                 derivation_path,
                 created_date);
    }

    println!("\n📈 Summary:");
    println!("   Total Subwallets: {}", subwallets.len());
    println!("   Address Group: {} (Index: {})", args.address_group, address_group.address_group_index);
    println!("   Base Wallet: {} ({})", args.wallet, base_wallet.blockchain);
    println!("   Wallet Group Account Index: {}", wallet_group.account_index);

    // Show derivation path pattern
    if let Some(first_subwallet) = subwallets.first() {
        if let Some(path) = &first_subwallet.derivation_path {
            println!("   Derivation Pattern: {}", path);

            // Extract and explain the components
            if let Some(parts) = extract_derivation_components(path) {
                println!("   Path Breakdown:");
                println!("     - Purpose: {} (BIP-44)", parts.0);
                println!("     - Coin Type: {} ({})", parts.1, base_wallet.blockchain);
                println!("     - Account: {} (Wallet Group)", parts.2);
                println!("     - Change: {} (Address Group)", parts.3);
                println!("     - Address Index: {} (Subwallet)", parts.4);
            }
        }
    }

    println!("\n💡 Next steps:");
    println!("   • Add more subwallets: wallet-backup add-subwallet --account \"{}\" --wallet-group \"{}\" --wallet \"{}\" --address-group \"{}\" --name \"<subwallet-name>\"", args.account, args.wallet_group, args.wallet, args.address_group);
    println!("   • Show subwallet details: wallet-backup show-wallet --address \"<address>\" --include-sensitive");
    println!("   • Create new address group: wallet-backup add-address-group --account \"{}\" --wallet-group \"{}\" --wallet \"{}\" --name \"<new-group-name>\"", args.account, args.wallet_group, args.wallet);

    Ok(())
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

fn extract_derivation_components(path: &str) -> Option<(String, String, String, String, String)> {
    // Parse derivation path like "m/44'/0'/0'/0/1"
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 6 && parts[0] == "m" {
        Some((
            parts[1].to_string(), // purpose (44')
            parts[2].to_string(), // coin type (0')
            parts[3].to_string(), // account (0')
            parts[4].to_string(), // change (0)
            parts[5].to_string(), // address index (1)
        ))
    } else {
        None
    }
}