use anyhow::{Result, Context};
use clap::Args;
use crate::database::Database;

#[derive(Args)]
pub struct CreateWalletGroupArgs {
    #[arg(long, help = "Name of the master account")]
    pub account: String,
    #[arg(long, help = "Name for the new wallet group")]
    pub name: String,
    #[arg(long, help = "Optional description for the wallet group")]
    pub description: Option<String>,
}

pub fn execute(args: CreateWalletGroupArgs, db: &Database) -> Result<()> {
    println!("Creating wallet group: {}", args.name);
    println!("Master account: {}", args.account);

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
                println!("\n💡 Use one of the existing account names or create a new master account.");
            }
            return Ok(());
        }
    };

    println!("✓ Master account found (ID: {})", master_account.id.unwrap_or(-1));

    // Create wallet group
    let (group_id, account_index) = db.create_wallet_group(
        master_account.id.unwrap(),
        &args.name,
        args.description.as_deref(),
    ).context("Failed to create wallet group")?;

    // Success message
    println!("\n🎉 Wallet group created successfully!");
    println!("   Group Name: {}", args.name);
    println!("   Group ID: {}", group_id);
    println!("   Master Account: {}", args.account);
    println!("   Account Index: {} (auto-assigned)", account_index);

    if let Some(desc) = &args.description {
        println!("   Description: {}", desc);
    }

    println!("\n💡 Next steps:");
    println!("   1. Add blockchains: wallet-backup add-blockchain --account \"{}\" --wallet-group \"{}\" --blockchains \"bitcoin,ethereum\"", args.account, args.name);
    println!("   2. List wallet groups: wallet-backup list-wallet-groups --account \"{}\"", args.account);
    println!("   3. Show group details: wallet-backup show-wallet-group --account \"{}\" --group \"{}\"", args.account, args.name);

    Ok(())
}