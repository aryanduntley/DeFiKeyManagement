use anyhow::{Result, Context};
use crate::database::Database;

pub fn execute(db: &Database) -> Result<()> {
    println!("Wallet Groups:");
    println!("{:-<80}", "");

    // Get all wallet groups
    let groups = db.get_all_wallet_groups()
        .context("Failed to retrieve wallet groups")?;

    if groups.is_empty() {
        println!("No wallet groups found.");
        println!("\nTo create a wallet group, use:");
        println!("  wallet-backup import-multi --mnemonic \"your words...\" --group-name \"MyWallet\"");
        return Ok(());
    }

    // Print header
    println!(
        "{:<25} {:<15} {:<8} {:<35} {}",
        "Group Name", "Blockchains", "Wallets", "Created", "Description"
    );
    println!("{:-<80}", "");

    // Print each group
    for group in &groups {
        let created = group.created_at.format("%Y-%m-%d %H:%M").to_string();
        let blockchains_display = if group.blockchains.len() <= 3 {
            group.blockchains.join(", ")
        } else {
            format!("{}, +{} more",
                    group.blockchains[..2].join(", "),
                    group.blockchains.len() - 2)
        };

        let description = group.description
            .as_deref()
            .unwrap_or("(no description)")
            .chars()
            .take(30)
            .collect::<String>();

        println!(
            "{:<25} {:<15} {:<8} {:<35} {}",
            truncate_string(&group.name, 24),
            truncate_string(&blockchains_display, 14),
            group.wallet_count,
            created,
            description
        );
    }

    println!("{:-<80}", "");
    println!("Total: {} wallet group(s)", groups.len());

    if !groups.is_empty() {
        println!("\nUse 'wallet-backup show-group \"<group-name>\"' to see wallets in a specific group.");
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