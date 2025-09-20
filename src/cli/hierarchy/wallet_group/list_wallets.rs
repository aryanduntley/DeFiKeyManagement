use anyhow::{Result, Context};
use clap::Args;
use crate::database::Database;

#[derive(Args)]
pub struct ListWalletsArgs {
    #[arg(long, help = "Name of the master account")]
    pub account: String,
    #[arg(long, help = "Name of the wallet group")]
    pub wallet_group: String,
}

pub fn execute(args: ListWalletsArgs, db: &Database) -> Result<()> {
    println!("📋 Listing wallets in wallet group");
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
            println!("\n❌ Wallet group '{}' not found in account '{}'.", args.wallet_group, args.account);
            println!("   Use 'wallet-backup list-wallet-groups --account \"{}\"' to see available groups.", args.account);
            return Ok(());
        }
    };

    // Get wallets in this wallet group (these are child private keys, address_group_id = NULL)
    // According to the hierarchy: wallets are child private keys that belong to wallet groups
    let wallets = db.get_wallets_by_wallet_group(wallet_group.id.unwrap())
        .context("Failed to get wallets for wallet group")?;

    if wallets.is_empty() {
        println!("\n📝 No wallets found in this wallet group.");
        println!("   Add a wallet: wallet-backup add-wallet --account \"{}\" --wallet-group \"{}\" --blockchain \"bitcoin\" --name \"MyWallet\"", args.account, args.wallet_group);
        return Ok(());
    }

    println!("\n💰 Wallets ({} total):", wallets.len());
    println!("   {:<20} {:<15} {:<45} {:<20} {:<12}",
             "Wallet Name", "Blockchain", "Address", "Derivation Path", "Created");
    println!("   {}", "─".repeat(120));

    for wallet in &wallets {
        let wallet_name = wallet.label.as_deref().unwrap_or("(unnamed)");
        let derivation_path = wallet.derivation_path.as_deref().unwrap_or("N/A");
        let created_date = wallet.created_at.format("%Y-%m-%d").to_string();

        println!("   {:<20} {:<15} {:<45} {:<20} {:<12}",
                 truncate_string(wallet_name, 18),
                 wallet.blockchain,
                 truncate_string(&wallet.address, 43),
                 derivation_path,
                 created_date);
    }

    println!("\n📈 Summary:");
    println!("   Total Wallets: {}", wallets.len());
    println!("   Account Index: {}", wallet_group.account_index);

    println!("\n💡 Next steps:");
    println!("   • View wallet details: wallet-backup show-wallet --account \"{}\" --wallet-group \"{}\" --wallet \"<wallet-name>\"", args.account, args.wallet_group);
    println!("   • Add subwallets: wallet-backup add-subwallet --account \"{}\" --wallet-group \"{}\" --wallet \"<wallet-name>\" --address-group \"<group-name>\" --name \"<subwallet-name>\"", args.account, args.wallet_group);
    println!("   • List address groups: wallet-backup list-address-groups --account \"{}\" --wallet-group \"{}\" --wallet \"<wallet-name>\"", args.account, args.wallet_group);

    Ok(())
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}