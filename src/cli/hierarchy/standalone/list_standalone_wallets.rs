use anyhow::{Result, Context};
use clap::Args;
use crate::database::Database;

#[derive(Args)]
pub struct ListStandaloneWalletsArgs {
    // No arguments needed - lists all standalone wallets
}

pub fn execute(_args: ListStandaloneWalletsArgs, db: &Database) -> Result<()> {
    println!("📋 Listing standalone wallets (imported from private keys)");

    // Get all standalone wallets (wallet_group_id = NULL, address_group_id = NULL)
    let wallets = db.get_standalone_wallets()
        .context("Failed to get standalone wallets")?;

    if wallets.is_empty() {
        println!("\n📝 No standalone wallets found.");
        println!("   Import a standalone wallet: wallet-backup add-standalone-wallet --private-key \"<key>\" --blockchain \"bitcoin\" --name \"MyImportedWallet\"");
        return Ok(());
    }

    println!("\n💰 Standalone Wallets ({} total):", wallets.len());
    println!("   {:<20} {:<15} {:<45} {:<15} {:<12}",
             "Wallet Name", "Blockchain", "Address", "Source", "Created");
    println!("   {}", "─".repeat(115));

    for wallet in &wallets {
        let wallet_name = wallet.label.as_deref().unwrap_or("(unnamed)");
        let source_type = &wallet.source_type;
        let created_date = wallet.created_at.format("%Y-%m-%d").to_string();

        println!("   {:<20} {:<15} {:<45} {:<15} {:<12}",
                 truncate_string(wallet_name, 18),
                 wallet.blockchain,
                 truncate_string(&wallet.address, 43),
                 source_type,
                 created_date);
    }

    // Group by blockchain for summary
    let mut blockchain_counts = std::collections::HashMap::new();
    for wallet in &wallets {
        *blockchain_counts.entry(&wallet.blockchain).or_insert(0) += 1;
    }

    println!("\n📈 Summary:");
    println!("   Total Standalone Wallets: {}", wallets.len());
    println!("   Blockchains:");
    for (blockchain, count) in blockchain_counts.iter() {
        println!("     {}: {} wallet{}", blockchain, count, if *count == 1 { "" } else { "s" });
    }

    println!("\n💡 Next steps:");
    println!("   • View wallet details: wallet-backup show-standalone-wallet --name \"<wallet-name>\"");
    println!("   • Import more wallets: wallet-backup add-standalone-wallet --private-key \"<key>\" --blockchain \"<blockchain>\" --name \"<name>\"");
    println!("   • Remove wallet: wallet-backup remove-standalone-wallet --name \"<wallet-name>\" --private-key \"<verification-key>\"");

    println!("\n⚠️  Note: Standalone wallets are not part of any hierarchical structure and cannot derive subwallets.");

    Ok(())
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}