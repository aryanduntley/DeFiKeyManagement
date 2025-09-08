use anyhow::{Result, bail};
use std::fs::File;
use std::io::Write;
use crate::cli::ExportArgs;
use crate::database::Database;

pub fn execute(args: ExportArgs, db: &Database) -> Result<()> {
    let wallets = if let Some(address) = &args.address {
        match db.get_wallet_by_address(address)? {
            Some(wallet) => vec![wallet],
            None => {
                println!("Wallet not found: {}", address);
                return Ok(());
            }
        }
    } else if let Some(label) = &args.label {
        match db.get_wallet_by_label(label)? {
            Some(wallet) => vec![wallet],
            None => {
                println!("Wallet not found: {}", label);
                return Ok(());
            }
        }
    } else {
        db.get_all_wallets()?
    };
    
    if wallets.is_empty() {
        println!("No wallets to export.");
        return Ok(());
    }
    
    let export_data = match args.format.as_str() {
        "json" => export_json(&wallets, args.include_sensitive)?,
        "csv" => export_csv(&wallets, args.include_sensitive)?,
        _ => bail!("Unsupported export format: {}. Use 'json' or 'csv'", args.format),
    };
    
    match args.output {
        Some(output_path) => {
            let mut file = File::create(&output_path)?;
            file.write_all(export_data.as_bytes())?;
            println!("Exported {} wallets to: {}", wallets.len(), output_path);
        },
        None => {
            println!("{}", export_data);
        }
    }
    
    Ok(())
}

fn export_json(wallets: &[crate::database::WalletRecord], include_sensitive: bool) -> Result<String> {
    if include_sensitive {
        Ok(serde_json::to_string_pretty(wallets)?)
    } else {
        let safe_wallets: Vec<_> = wallets.iter().map(|w| {
            let mut safe = w.clone();
            safe.private_key = "[REDACTED]".to_string();
            safe.mnemonic = safe.mnemonic.as_ref().map(|_| "[REDACTED]".to_string());
            safe.passphrase = safe.passphrase.as_ref().map(|_| "[REDACTED]".to_string());
            safe
        }).collect();
        Ok(serde_json::to_string_pretty(&safe_wallets)?)
    }
}

fn export_csv(wallets: &[crate::database::WalletRecord], include_sensitive: bool) -> Result<String> {
    let mut csv = String::new();
    
    if include_sensitive {
        csv.push_str("label,blockchain,address,public_key,private_key,mnemonic,passphrase,derivation_path,account,address_index,source_type,explorer_url,imported_at,notes\n");
        for wallet in wallets {
            csv.push_str(&format!("{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
                wallet.label.as_deref().unwrap_or(""),
                wallet.blockchain,
                wallet.address,
                wallet.public_key.as_deref().unwrap_or(""),
                wallet.private_key,
                wallet.mnemonic.as_deref().unwrap_or(""),
                wallet.passphrase.as_deref().unwrap_or(""),
                wallet.derivation_path,
                wallet.account.map(|a| a.to_string()).unwrap_or_default(),
                wallet.address_index.map(|i| i.to_string()).unwrap_or_default(),
                wallet.source_type,
                wallet.explorer_url.as_deref().unwrap_or(""),
                wallet.imported_at.format("%Y-%m-%d %H:%M:%S UTC"),
                wallet.notes.as_deref().unwrap_or("")
            ));
        }
    } else {
        csv.push_str("label,blockchain,address,derivation_path,account,address_index,source_type,explorer_url,imported_at,notes\n");
        for wallet in wallets {
            csv.push_str(&format!("{},{},{},{},{},{},{},{},{},{}\n",
                wallet.label.as_deref().unwrap_or(""),
                wallet.blockchain,
                wallet.address,
                wallet.derivation_path,
                wallet.account.map(|a| a.to_string()).unwrap_or_default(),
                wallet.address_index.map(|i| i.to_string()).unwrap_or_default(),
                wallet.source_type,
                wallet.explorer_url.as_deref().unwrap_or(""),
                wallet.imported_at.format("%Y-%m-%d %H:%M:%S UTC"),
                wallet.notes.as_deref().unwrap_or("")
            ));
        }
    }
    
    Ok(csv)
}