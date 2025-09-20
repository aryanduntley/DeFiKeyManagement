use anyhow::{Result, Context};
use clap::Args;
use crate::database::Database;

#[derive(Args)]
pub struct ShowWalletGroupArgs {
    #[arg(long, help = "Name of the master account")]
    pub account: String,
    #[arg(long, help = "Name of the wallet group to show")]
    pub group_name: String,
    #[arg(long, help = "Include sensitive information (private keys)")]
    pub include_sensitive: bool,
}

pub fn execute(args: ShowWalletGroupArgs, db: &Database) -> Result<()> {
    println!("🔍 Wallet Group Details");
    println!("Master Account: {}", args.account);
    println!("Group Name: {}", args.group_name);

    // Get master account by name
    let master_account = match db.get_master_account_by_name(&args.account)? {
        Some(account) => account,
        None => {
            println!("\n❌ Master account '{}' not found.", args.account);
            return Ok(());
        }
    };

    // Get wallet group by name
    let wallet_group = match db.get_wallet_group_by_name(master_account.id.unwrap(), &args.group_name)? {
        Some(group) => group,
        None => {
            println!("\n❌ Wallet group '{}' not found in account '{}'.", args.group_name, args.account);
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

    // Get base wallets in this wallet group (address_group_id = NULL)
    // These are the child private keys that belong directly to the wallet group
    let wallets = db.get_wallets_by_wallet_group(wallet_group.id.unwrap())
        .context("Failed to get wallets for wallet group")?;

    if wallets.is_empty() {
        println!("\n📝 No wallets added to this group yet.");
        println!("   Add a wallet: wallet-backup add-wallet --account \"{}\" --wallet-group \"{}\" --blockchain \"bitcoin\" --name \"MyWallet\"", args.account, args.group_name);
        return Ok(());
    }

    println!("\n💰 Wallets ({}):", wallets.len());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    for (i, wallet) in wallets.iter().enumerate() {
        let wallet_name = wallet.label.as_deref().unwrap_or("(unnamed)");
        let derivation_path = wallet.derivation_path.as_deref().unwrap_or("N/A");

        println!("   {}. 📱 {}", i + 1, wallet_name);
        println!("      🔗 Blockchain: {}", wallet.blockchain);
        println!("      📍 Address: {}", wallet.address);
        println!("      🛤️  Path: {}", derivation_path);

        if args.include_sensitive {
            println!("      🔒 Private Key: {}", wallet.private_key);
        } else {
            println!("      🔒 Private Key: (use --include-sensitive to view)");
        }

        if i < wallets.len() - 1 {
            println!();
        }
    }


    println!("\n💡 Next steps:");
    println!("   • Add more wallets: wallet-backup add-wallet --account \"{}\" --wallet-group \"{}\" --blockchain \"<blockchain>\" --name \"<wallet-name>\"", args.account, args.group_name);
    println!("   • List wallets only: wallet-backup list-wallets --account \"{}\" --wallet-group \"{}\"", args.account, args.group_name);
    println!("   • Add address groups: wallet-backup add-address-group --account \"{}\" --wallet-group \"{}\" --wallet \"<wallet-name>\" --name \"<group-name>\"", args.account, args.group_name);
    println!("   • View subwallets: wallet-backup list-subwallets --account \"{}\" --wallet-group \"{}\" --wallet \"<wallet-name>\" --address-group \"<group-name>\"", args.account, args.group_name);

    Ok(())
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}