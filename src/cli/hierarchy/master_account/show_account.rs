use anyhow::Result;
use clap::Args;
use crate::database::Database;

#[derive(Args)]
pub struct ShowAccountArgs {
    #[arg(long, help = "Name of the master account to show")]
    pub account: String,
    #[arg(long, help = "Include sensitive information (mnemonic phrase)")]
    pub include_sensitive: bool,
}

pub fn execute(args: ShowAccountArgs, db: &Database) -> Result<()> {
    println!("🔍 Master Account Details");
    println!("Account Name: {}", args.account);

    // Get master account by name
    let master_account = match db.get_master_account_by_name(&args.account)? {
        Some(account) => account,
        None => {
            println!("\n❌ Master account '{}' not found.", args.account);
            println!("   Use 'wallet-backup list-accounts' to see available accounts.");
            return Ok(());
        }
    };

    // Display master account summary
    println!("\n📊 Account Summary:");
    println!("   Account ID: {}", master_account.id.unwrap_or(-1));
    println!("   Account Name: {}", master_account.name);
    println!("   Next Account Index: {}", master_account.next_account_index);
    println!("   Created: {}", master_account.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("   Updated: {}", master_account.updated_at.format("%Y-%m-%d %H:%M:%S UTC"));

    if let Some(ref passphrase) = master_account.passphrase {
        if !passphrase.is_empty() {
            println!("   Passphrase: ✓ (configured - view with --include-sensitive)");
        } else {
            println!("   Passphrase: (none)");
        }
    } else {
        println!("   Passphrase: (none)");
    }

    // Get wallet groups count
    let wallet_groups = db.list_wallet_groups(master_account.id.unwrap())?;
    println!("   Wallet Groups: {}", wallet_groups.len());

    // Get total wallets count across all groups
    let mut total_wallets = 0;
    for group in &wallet_groups {
        let wallets = db.get_wallets_by_wallet_group(group.id)?;
        total_wallets += wallets.len();
    }
    println!("   Total Wallets: {}", total_wallets);

    // Show wallet groups summary
    if !wallet_groups.is_empty() {
        println!("\n📁 Wallet Groups:");
        println!("   {:<20} {:<8} {:<12} {:<20}",
                 "Group Name", "Wallets", "Account Idx", "Created");
        println!("   {}", "─".repeat(70));

        for group in &wallet_groups {
            let group_wallets = db.get_wallets_by_wallet_group(group.id)?;
            let created_date = group.created_at.format("%Y-%m-%d").to_string();

            println!("   {:<20} {:<8} {:<12} {:<20}",
                     truncate_string(&group.name, 18),
                     group_wallets.len(),
                     group.account_index,
                     created_date);
        }
    }

    // Show sensitive information if requested
    if args.include_sensitive {
        println!("\n🔒 Sensitive Information:");
        println!("   Mnemonic Phrase: {}", master_account.mnemonic);

        if let Some(ref passphrase) = master_account.passphrase {
            if !passphrase.is_empty() {
                println!("   Passphrase: {}", passphrase);
            } else {
                println!("   Passphrase: (none)");
            }
        } else {
            println!("   Passphrase: (none)");
        }

        println!("   Master Private Key: {}", master_account.master_private_key);

        println!("\n⚠️  SECURITY WARNING:");
        println!("   • Never share your mnemonic phrase or private key with anyone");
        println!("   • Store this information in a secure location");
        println!("   • Anyone with access to this data can control all derived wallets");

        println!("\n💡 Import Instructions:");
        println!("   • Most wallets: Use the 12-24 word mnemonic phrase above");
        if let Some(ref passphrase) = master_account.passphrase {
            if !passphrase.is_empty() {
                println!("   • Include passphrase: \"{}\" (25th word/additional security)", passphrase);
            }
        }
        println!("   • Derivation paths: Standard BIP-44 (wallets auto-detect)");
        println!("   • Account index: Start from {} and increment as needed", master_account.next_account_index - 1);
    } else {
        println!("\n💡 To view mnemonic phrase for wallet import, use: --include-sensitive");
    }

    println!("\n💡 Next steps:");
    println!("   • View wallet groups: wallet-backup list-wallet-groups --account \"{}\"", args.account);
    if !wallet_groups.is_empty() {
        println!("   • View group details: wallet-backup show-wallet-group --account \"{}\" --group \"<group-name>\"", args.account);
    } else {
        println!("   • Create wallet group: wallet-backup add-wallet-group --account \"{}\" --name \"<group-name>\"", args.account);
    }
    println!("   • Export for backup: wallet-backup export --account \"{}\"", args.account);

    Ok(())
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}