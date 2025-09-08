use anyhow::{Result, bail};
use crate::cli::TagArgs;
use crate::database::Database;

pub fn execute(args: TagArgs, db: &Database) -> Result<()> {
    let wallet_address = match (&args.address, &args.current_label) {
        (Some(address), None) => {
            // Update by address
            address.clone()
        },
        (None, Some(current_label)) => {
            // Find wallet by current label and get its address
            match db.get_wallet_by_label(current_label)? {
                Some(wallet) => wallet.address,
                None => {
                    println!("Wallet not found with label: {}", current_label);
                    return Ok(());
                }
            }
        },
        (Some(_), Some(_)) => bail!("Cannot specify both --address and --current-label"),
        (None, None) => bail!("Must specify either --address or --current-label"),
    };
    
    // Check if wallet exists
    let wallet = match db.get_wallet_by_address(&wallet_address)? {
        Some(w) => w,
        None => {
            println!("Wallet not found: {}", wallet_address);
            return Ok(());
        }
    };
    
    println!("Updating label for wallet:");
    println!("  Address: {}", wallet.address);
    println!("  Current label: {}", wallet.label.as_deref().unwrap_or("(no label)"));
    println!("  New label: {}", args.label);
    
    let updated = db.update_wallet_label(&wallet_address, &args.label)?;
    
    if updated {
        println!("Wallet label updated successfully.");
    } else {
        println!("Failed to update wallet label.");
    }
    
    Ok(())
}