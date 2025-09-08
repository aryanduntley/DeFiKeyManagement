use anyhow::Result;
use crate::cli::SearchArgs;
use crate::database::Database;

pub fn execute(args: SearchArgs, db: &Database) -> Result<()> {
    let wallets = db.search_wallets(&args.term, args.blockchain.as_deref())?;
    
    if wallets.is_empty() {
        println!("No wallets found matching: {}", args.term);
        if let Some(blockchain) = &args.blockchain {
            println!("  (searched in {} blockchain only)", blockchain);
        }
        return Ok(());
    }
    
    println!("Found {} wallet(s) matching '{}':", wallets.len(), args.term);
    println!();
    println!("{:<20} {:<12} {:<42} {:<25}", "Label", "Blockchain", "Address", "Path");
    println!("{}", "-".repeat(100));
    
    for wallet in wallets {
        let label = wallet.label.as_deref().unwrap_or("(no label)");
        let address_display = if wallet.address.len() > 40 {
            format!("{}...{}", &wallet.address[..20], &wallet.address[wallet.address.len()-20..])
        } else {
            wallet.address.clone()
        };
        
        println!("{:<20} {:<12} {:<42} {:<25}", 
                 label, 
                 wallet.blockchain, 
                 address_display,
                 wallet.derivation_path);
    }
    
    Ok(())
}