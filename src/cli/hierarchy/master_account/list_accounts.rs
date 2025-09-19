use anyhow::Result;
use clap::Args;
use crate::database::Database;

#[derive(Args)]
pub struct ListAccountsArgs {
    // No arguments needed for list-accounts command
}

pub fn execute(_args: ListAccountsArgs, db: &Database) -> Result<()> {
    println!("📋 Accounts");

    let master_accounts = db.list_master_accounts()?;

    if master_accounts.is_empty() {
        println!("\n   No accounts found.");
        println!("   Create one with: wallet-backup create-account --account-name \"YourAccountName\" --mnemonic \"your mnemonic phrase\"");
        return Ok(());
    }

    println!("   Found {} account(s):\n", master_accounts.len());

    // Print header
    println!("   {:<4} {:<25} {:<12} {:<15} {:<12}",
        "ID", "Account Name", "Groups", "Addresses", "Created");
    println!("   {}", "─".repeat(70));

    // Print each master account
    for account in master_accounts {
        let created_date = account.created_at.format("%Y-%m-%d").to_string();

        println!("   {:<4} {:<25} {:<12} {:<15} {:<12}",
            account.id,
            account.name,
            account.wallet_group_count,
            account.total_addresses,
            created_date
        );
    }

    println!("\n💡 Next steps:");
    println!("   • Create wallet groups: wallet-backup create-wallet-group --account \"<account-name>\" --name \"<group-name>\"");
    println!("   • Show specific account details: wallet-backup show-hierarchy --account \"<account-name>\"");

    Ok(())
}