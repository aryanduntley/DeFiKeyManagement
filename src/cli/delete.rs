use anyhow::{Result, bail};
use std::io::{self, Write};
use crate::cli::DeleteArgs;
use crate::database::Database;

pub fn execute(args: DeleteArgs, db: &Database) -> Result<()> {
    let (wallet, identifier) = match (&args.address, &args.label) {
        (Some(address), None) => {
            let wallet = db.get_wallet_by_address(address)?;
            (wallet, address.clone())
        },
        (None, Some(label)) => {
            let wallet = db.get_wallet_by_label(label)?;
            (wallet, label.clone())
        },
        (Some(_), Some(_)) => bail!("Cannot specify both --address and --label"),
        (None, None) => bail!("Must specify either --address or --label"),
    };
    
    let wallet = match wallet {
        Some(w) => w,
        None => {
            println!("Wallet not found: {}", identifier);
            return Ok(());
        }
    };
    
    // Show wallet info before deletion
    println!("Wallet to delete:");
    println!("  Label: {}", wallet.label.as_deref().unwrap_or("(no label)"));
    println!("  Blockchain: {}", wallet.blockchain);
    println!("  Address: {}", wallet.address);
    
    // Confirmation unless --force is used
    if !args.force {
        print!("Are you sure you want to delete this wallet? [y/N]: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let confirmed = input.trim().to_lowercase();
        if confirmed != "y" && confirmed != "yes" {
            println!("Deletion cancelled.");
            return Ok(());
        }
    }
    
    // Delete wallet
    let deleted = if args.address.is_some() {
        db.delete_wallet_by_address(&wallet.address)?
    } else {
        db.delete_wallet_by_label(wallet.label.as_deref().unwrap())?
    };
    
    if deleted {
        println!("Wallet deleted successfully.");
    } else {
        println!("Failed to delete wallet (may have already been deleted).");
    }
    
    Ok(())
}