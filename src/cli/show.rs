use anyhow::{Result, bail};
use crate::cli::ShowArgs;
use crate::database::Database;

pub fn execute(args: ShowArgs, db: &Database) -> Result<()> {
    let wallet = match (&args.address, &args.label) {
        (Some(address), None) => db.get_wallet_by_address(address)?,
        (None, Some(label)) => db.get_wallet_by_label(label)?,
        (Some(_), Some(_)) => bail!("Cannot specify both --address and --label"),
        (None, None) => bail!("Must specify either --address or --label"),
    };
    
    match wallet {
        Some(w) => {
            println!("Wallet Details:");
            println!("---------------");
            println!("Label: {}", w.label.as_deref().unwrap_or("(no label)"));
            println!("Blockchain: {}", w.blockchain);
            println!("Address: {}", w.address);
            println!("Derivation Path: {}", w.derivation_path);
            if let Some(account) = w.account {
                println!("Account: {}", account);
            }
            if let Some(index) = w.address_index {
                println!("Address Index: {}", index);
            }
            println!("Source Type: {}", w.source_type);
            println!("Imported At: {}", w.imported_at.format("%Y-%m-%d %H:%M:%S UTC"));
            
            if let Some(explorer_url) = &w.explorer_url {
                println!("Explorer: {}", explorer_url);
            }
            
            if let Some(notes) = &w.notes {
                println!("Notes: {}", notes);
            }
            
            if args.include_sensitive {
                println!("\nSENSITIVE DATA:");
                println!("Private Key: {}", w.private_key);
                if let Some(mnemonic) = &w.mnemonic {
                    println!("Mnemonic: {}", mnemonic);
                }
                if let Some(passphrase) = &w.passphrase {
                    println!("Passphrase: {}", passphrase);
                }
                if let Some(pubkey) = &w.public_key {
                    println!("Public Key: {}", pubkey);
                }
            } else {
                println!("\nUse --include-sensitive to show private key and mnemonic");
            }
        },
        None => {
            let search_term = args.address.as_deref().or(args.label.as_deref()).unwrap();
            println!("Wallet not found: {}", search_term);
        }
    }
    
    Ok(())
}