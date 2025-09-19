use anyhow::{Result, Context};
use clap::Args;
use crate::database::Database;

#[derive(Args)]
pub struct ListWalletGroupsArgs {
    #[arg(long, help = "Name of the master account")]
    pub account: String,
}

pub fn execute(args: ListWalletGroupsArgs, db: &Database) -> Result<()> {
    println!("📋 Wallet Groups for Master Account: {}", args.account);

    // Get master account by name
    let master_account = match db.get_master_account_by_name(&args.account)? {
        Some(account) => account,
        None => {
            println!("❌ Master account '{}' not found.", args.account);
            println!("   Available accounts:");

            let accounts = db.list_master_accounts().context("Failed to list master accounts")?;
            if accounts.is_empty() {
                println!("   (none)");
                println!("\n💡 Create an account first: wallet-backup create-account --account-name \"{}\" --mnemonic \"...\"", args.account);
            } else {
                for account in accounts {
                    println!("   - {}", account.name);
                }
                println!("\n💡 Use one of the existing account names.");
            }
            return Ok(());
        }
    };

    // List wallet groups for this master account
    let wallet_groups = db.list_wallet_groups(master_account.id.unwrap())
        .context("Failed to list wallet groups")?;

    if wallet_groups.is_empty() {
        println!("   No wallet groups found for account '{}'.", args.account);
        println!("\n💡 Create a wallet group first:");
        println!("   wallet-backup create-wallet-group --account \"{}\" --name \"PersonalWallet\"", args.account);
        return Ok(());
    }

    println!("   Found {} wallet group(s):\n", wallet_groups.len());

    // Table header
    println!("   {:3} {:<20} {:<12} {:<10} {:<15} {:<12}",
             "ID", "Group Name", "Account Idx", "Addresses", "Address Groups", "Created");
    println!("   {}", "─".repeat(80));

    // Table rows
    for group in &wallet_groups {
        println!("   {:3} {:<20} {:<12} {:<10} {:<15} {:<12}",
                 group.id,
                 truncate_string(&group.name, 18),
                 group.account_index,
                 group.total_addresses,
                 group.address_group_count,
                 group.created_at.format("%Y-%m-%d").to_string()
        );
    }

    println!("\n💡 Next steps:");
    for group in wallet_groups.iter().take(3) {
        println!("   • Show '{}' details: wallet-backup show-wallet-group --account \"{}\" --group \"{}\"",
                 group.name, args.account, group.name);
    }
    if wallet_groups.len() > 3 {
        println!("   • ... and {} more groups", wallet_groups.len() - 3);
    }

    Ok(())
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}