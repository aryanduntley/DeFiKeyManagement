use anyhow::{Result, Context};
use crate::cli::RenameGroupArgs;
use crate::database::Database;

pub fn execute(args: RenameGroupArgs, db: &Database) -> Result<()> {
    println!("Renaming wallet group...");
    println!("Old name: {}", args.old_name);
    println!("New name: {}", args.new_name);

    // Check if the old group exists
    let old_group = db.get_wallet_group_by_name(&args.old_name)?
        .ok_or_else(|| anyhow::anyhow!("Wallet group '{}' not found", args.old_name))?;

    println!("Found group with {} wallets across {} blockchains",
        {
            let wallets = db.get_wallets_by_group_id(old_group.id.unwrap())?;
            wallets.len()
        },
        old_group.blockchains.len());

    // Prompt for confirmation unless --force is used
    if !args.force {
        println!("\nThis will rename the group and all associated wallet labels.");
        println!("Current wallets in group:");

        let wallets = db.get_wallets_by_group_id(old_group.id.unwrap())?;
        for wallet in &wallets {
            if let Some(label) = &wallet.label {
                println!("  - {}: {}", wallet.blockchain, label);
            }
        }

        print!("\nProceed with rename? [y/N]: ");
        use std::io::{self, Write};
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).context("Failed to read input")?;

        let input = input.trim().to_lowercase();
        if input != "y" && input != "yes" {
            println!("Rename cancelled.");
            return Ok(());
        }
    }

    // Perform the rename
    let success = db.rename_wallet_group(&args.old_name, &args.new_name)
        .context("Failed to rename wallet group")?;

    if !success {
        println!("❌ Group '{}' not found.", args.old_name);
        return Ok(());
    }

    // Update wallet labels to reflect the new group name
    let wallets = db.get_wallets_by_group_id(old_group.id.unwrap())?;
    let mut updated_labels = 0;

    for wallet in wallets {
        if let Some(old_label) = &wallet.label {
            // Check if the label follows the pattern "GroupName_blockchain"
            if old_label.starts_with(&format!("{}_", args.old_name)) {
                let blockchain_part = &old_label[args.old_name.len() + 1..];
                let new_label = format!("{}_{}", args.new_name, blockchain_part);

                if db.update_wallet_label(&wallet.address, &new_label)? {
                    updated_labels += 1;
                }
            }
        }
    }

    println!("✅ Successfully renamed group '{}' to '{}'", args.old_name, args.new_name);
    if updated_labels > 0 {
        println!("✅ Updated {} wallet labels to match new group name", updated_labels);
    }

    println!("\nUse 'wallet-backup show-group \"{}\"' to view the renamed group.", args.new_name);

    Ok(())
}