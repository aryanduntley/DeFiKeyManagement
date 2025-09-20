use anyhow::{Result, Context};
use clap::Args;
use crate::database::Database;

#[derive(Args)]
pub struct ListAddressGroupsArgs {
    #[arg(long, help = "Name of the master account")]
    pub account: String,
    #[arg(long, help = "Name of the wallet group")]
    pub wallet_group: String,
    #[arg(long, help = "Name of the wallet (base wallet)")]
    pub wallet: String,
}

pub fn execute(args: ListAddressGroupsArgs, db: &Database) -> Result<()> {
    println!("📋 Listing address groups for wallet");
    println!("Account: {}", args.account);
    println!("Wallet Group: {}", args.wallet_group);
    println!("Wallet: {}", args.wallet);

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

    // Get the specific wallet (base wallet) by name within this wallet group
    let base_wallet = match db.get_wallet_by_name_in_group(wallet_group.id.unwrap(), &args.wallet)? {
        Some(wallet) => wallet,
        None => {
            println!("\n❌ Wallet '{}' not found in wallet group '{}'.", args.wallet, args.wallet_group);
            println!("   Use 'wallet-backup list-wallets --account \"{}\" --wallet-group \"{}\"' to see available wallets.", args.account, args.wallet_group);
            return Ok(());
        }
    };

    // Get address groups for this specific wallet
    let address_groups = db.list_address_groups_for_wallet(base_wallet.id.unwrap())
        .context("Failed to get address groups for wallet")?;

    if address_groups.is_empty() {
        println!("\n📝 No address groups found for this wallet.");
        println!("   Create an address group: wallet-backup add-address-group --account \"{}\" --wallet-group \"{}\" --wallet \"{}\" --name \"receiving\"", args.account, args.wallet_group, args.wallet);
        return Ok(());
    }

    println!("\n🗂️  Address Groups ({} total):", address_groups.len());
    println!("   {:<20} {:<15} {:<15} {:<15} {:<12}",
             "Group Name", "Blockchain", "Group Index", "Subwallets", "Created");
    println!("   {}", "─".repeat(85));

    for address_group in &address_groups {
        let created_date = address_group.created_at.format("%Y-%m-%d").to_string();

        println!("   {:<20} {:<15} {:<15} {:<15} {:<12}",
                 truncate_string(&address_group.name, 18),
                 address_group.blockchain,
                 address_group.address_group_index,
                 address_group.wallet_count,
                 created_date);
    }

    println!("\n📈 Summary:");
    println!("   Total Address Groups: {}", address_groups.len());
    println!("   Base Wallet: {} ({})", args.wallet, base_wallet.blockchain);
    println!("   Derivation Path: {}", base_wallet.derivation_path.as_deref().unwrap_or("N/A"));

    println!("\n💡 Next steps:");
    println!("   • View subwallets in group: wallet-backup list-subwallets --account \"{}\" --wallet-group \"{}\" --wallet \"{}\" --address-group \"<group-name>\"", args.account, args.wallet_group, args.wallet);
    println!("   • Add subwallets: wallet-backup add-subwallet --account \"{}\" --wallet-group \"{}\" --wallet \"{}\" --address-group \"<group-name>\" --name \"<subwallet-name>\"", args.account, args.wallet_group, args.wallet);
    println!("   • Create new group: wallet-backup add-address-group --account \"{}\" --wallet-group \"{}\" --wallet \"{}\" --name \"<new-group-name>\"", args.account, args.wallet_group, args.wallet);

    Ok(())
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}