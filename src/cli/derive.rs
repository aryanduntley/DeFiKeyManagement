use anyhow::{Result, Context};
use chrono::Utc;
use crate::cli::DeriveArgs;
use crate::database::{Database, WalletRecord};
use crate::blockchain::{SupportedBlockchain, get_blockchain_handler};
use crate::crypto::bip39_utils::{validate_mnemonic_phrase, normalize_mnemonic};

pub fn execute(args: DeriveArgs, db: &Database) -> Result<()> {
    println!("Deriving {} addresses for {} blockchain", args.count, args.blockchain);
    println!("Account: {}, Starting index: {}", args.account, args.index);
    
    // Parse blockchain
    let blockchain = SupportedBlockchain::from_str(&args.blockchain)
        .context("Unsupported blockchain")?;
    
    // Get blockchain handler
    let handler = get_blockchain_handler(&blockchain)
        .context("Failed to get blockchain handler")?;

    // Validate mnemonic
    let normalized_mnemonic = normalize_mnemonic(&args.mnemonic);
    validate_mnemonic_phrase(&normalized_mnemonic)
        .context("Invalid mnemonic phrase")?;

    println!("\nDeriving addresses...");
    let mut derived_wallets = Vec::new();
    
    // Derive multiple addresses
    for i in 0..args.count {
        let address_index = args.index + i;
        
        // Derive wallet keys for this index
        let wallet_keys = handler.derive_from_mnemonic(
            &normalized_mnemonic,
            args.passphrase.as_deref(),
            args.account,
            address_index,
            args.custom_path.as_deref(),
        ).context(format!("Failed to derive address at index {}", address_index))?;

        // Validate generated address
        if !handler.validate_address(&wallet_keys.address) {
            return Err(anyhow::anyhow!("Generated address failed validation at index {}", address_index));
        }

        // Check if wallet already exists
        let already_exists = db.get_wallet_by_address(&wallet_keys.address)?.is_some();
        
        if already_exists {
            println!("  {}: {} (already exists)", address_index, wallet_keys.address);
            continue;
        }

        // Generate explorer URL
        let explorer_url = blockchain.get_explorer_url(&wallet_keys.address);

        // Create wallet record
        let wallet_record = WalletRecord {
            id: None,
            label: Some(format!("{}_{}_{}", args.blockchain, args.account, address_index)),
            blockchain: args.blockchain.clone(),
            address: wallet_keys.address.clone(),
            address_with_checksum: wallet_keys.address_with_checksum.clone(),
            public_key: Some(wallet_keys.public_key.clone()),
            private_key: wallet_keys.private_key.clone(),
            mnemonic: Some(normalized_mnemonic.clone()),
            passphrase: args.passphrase.clone(),
            derivation_path: wallet_keys.derivation_path.clone(),
            account: Some(args.account),
            address_index: Some(address_index),
            source_type: "mnemonic".to_string(),
            explorer_url: Some(explorer_url.clone()),
            imported_at: Utc::now(),
            notes: None,
            additional_data: wallet_keys.additional_data.clone(),
            secondary_addresses: wallet_keys.secondary_addresses.clone(),
            group_id: None, // For now, no group assignment in basic derive
        };

        // Insert into database
        let wallet_id = db.insert_wallet(&wallet_record)
            .context(format!("Failed to save wallet {} to database", address_index))?;

        println!("  {}: {} (saved as ID: {})", address_index, wallet_keys.address, wallet_id);
        derived_wallets.push((wallet_id, wallet_keys));
    }
    
    // Summary
    if derived_wallets.is_empty() {
        println!("\nNo new addresses derived (all already existed).");
    } else {
        println!("\n✓ Successfully derived {} new addresses!", derived_wallets.len());
        println!("  Blockchain: {}", args.blockchain);
        println!("  Account: {}", args.account);
        println!("  Address Range: {} - {}", args.index, args.index + args.count - 1);
        
        if args.count <= 5 {
            println!("\nAddress Details:");
            for (wallet_id, keys) in &derived_wallets {
                println!("  ID {}: {}", wallet_id, keys.address);
            }
        }
        
        println!("\nUse 'wallet-backup list' to see all wallets.");
    }
    
    Ok(())
}