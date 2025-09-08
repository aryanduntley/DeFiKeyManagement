use anyhow::Result;
use crate::database::Database;

pub fn execute(db: &Database) -> Result<()> {
    let wallets = db.get_all_wallets()?;
    
    if wallets.is_empty() {
        println!("No wallets found. Use 'wallet-backup import' to add wallets.");
        return Ok(());
    }
    
    println!("{:<20} {:<12} {:<42} {:<25}", "Label", "Blockchain", "Address", "Path");
    println!("{}", "-".repeat(100));
    
    for wallet in &wallets {
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
    
    println!("\nTotal: {} wallets", wallets.len());
    Ok(())
}