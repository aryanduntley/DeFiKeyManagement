use anyhow::{Result, bail, Context};
use chrono::Utc;
use crate::cli::DeriveMultiArgs;
use crate::database::{Database, WalletRecord};
use crate::blockchain::{SupportedBlockchain, get_blockchain_handler};
use crate::crypto::bip39_utils::{validate_mnemonic_phrase, normalize_mnemonic};

pub fn execute(args: DeriveMultiArgs, db: &Database) -> Result<()> {
    println!("Adding blockchains to existing wallet group...");
    println!("Group: {}", args.group_name);

    // Get the existing wallet group
    let wallet_group = db.get_wallet_group_by_name(&args.group_name)?
        .ok_or_else(|| anyhow::anyhow!("Wallet group '{}' not found", args.group_name))?;

    println!("Found existing group with {} blockchains: {}",
        wallet_group.blockchains.len(),
        wallet_group.blockchains.join(", "));

    // Parse and validate new blockchains
    let new_blockchain_names: Vec<String> = args.blockchains.split(',')
        .map(|s| s.trim().to_lowercase())
        .collect();

    // Validate all requested blockchains are supported
    let supported_blockchains = SupportedBlockchain::validate_blockchains(&new_blockchain_names)
        .context("Blockchain validation failed")?;

    // Filter out blockchains that already exist in the group
    let existing_blockchains: std::collections::HashSet<String> = wallet_group.blockchains.iter()
        .map(|s| s.to_lowercase())
        .collect();

    let new_blockchains_filtered: Vec<SupportedBlockchain> = supported_blockchains.into_iter()
        .filter(|blockchain| !existing_blockchains.contains(&blockchain.to_string().to_lowercase()))
        .collect();

    if new_blockchains_filtered.is_empty() {
        println!("⚠️  All specified blockchains already exist in group '{}'", args.group_name);
        println!("Existing blockchains: {}", wallet_group.blockchains.join(", "));
        return Ok(());
    }

    let new_blockchain_names_filtered: Vec<String> = new_blockchains_filtered.iter()
        .map(|b| b.to_string().to_lowercase())
        .collect();

    println!("New blockchains to add: {}", new_blockchain_names_filtered.join(", "));

    // Add the new blockchains to the group
    let group_id = wallet_group.id.unwrap();
    db.add_blockchains_to_group(group_id, &new_blockchain_names_filtered)
        .context("Failed to add blockchains to group")?;

    // Now we need the mnemonic to derive wallets for the new blockchains
    let mnemonic = match &args.mnemonic {
        Some(m) => m.clone(),
        None => bail!("Mnemonic is required to derive wallets for new blockchains. Use --mnemonic parameter."),
    };
    let normalized_mnemonic = normalize_mnemonic(&mnemonic);
    validate_mnemonic_phrase(&normalized_mnemonic)
        .context("Invalid mnemonic phrase")?;

    // Verify mnemonic matches the group (security check)
    let mnemonic_hash = {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(normalized_mnemonic.as_bytes());
        hex::encode(hasher.finalize())
    };

    if mnemonic_hash != wallet_group.mnemonic_hash {
        bail!("Provided mnemonic does not match the mnemonic used for group '{}'", args.group_name);
    }

    // Derive wallets for each new blockchain
    let mut success_count = 0;
    let mut skipped_count = 0;
    let mut failed_blockchains = Vec::new();

    for blockchain in new_blockchains_filtered {
        let blockchain_name = blockchain.to_string().to_lowercase();

        print!("Deriving {} wallet... ", blockchain_name);

        match derive_wallet_for_blockchain(
            &blockchain,
            &normalized_mnemonic,
            &args,
            group_id,
            db
        ) {
            Ok(wallet_id) => {
                if let Some(id) = wallet_id {
                    println!("✓ Success (ID: {})", id);
                    success_count += 1;
                } else {
                    println!("⚠ Skipped (already exists)");
                    skipped_count += 1;
                }
            }
            Err(e) => {
                println!("✗ Failed: {}", e);
                failed_blockchains.push(blockchain_name);
            }
        }
    }

    // Summary
    println!("\n🎊 Blockchain extension complete!");
    println!("  Group: {}", args.group_name);
    println!("  ✅ Successfully added: {}", success_count);
    if skipped_count > 0 {
        println!("  ⚠️  Skipped (already exist): {}", skipped_count);
    }
    if !failed_blockchains.is_empty() {
        println!("  ❌ Failed: {}", failed_blockchains.join(", "));
    }

    if success_count > 0 {
        println!("\nUse 'wallet-backup show-group \"{}\"' to see all wallets in this group.", args.group_name);
    }

    Ok(())
}

fn derive_wallet_for_blockchain(
    blockchain: &SupportedBlockchain,
    mnemonic: &str,
    args: &DeriveMultiArgs,
    group_id: i64,
    db: &Database,
) -> Result<Option<i64>> {
    // Get blockchain handler
    let handler = get_blockchain_handler(blockchain)
        .context("Failed to get blockchain handler")?;

    // Derive wallet keys
    let wallet_keys = handler.derive_from_mnemonic(
        mnemonic,
        args.passphrase.as_deref(),
        args.account.unwrap_or(0),
        args.address_index.unwrap_or(0),
        None, // No custom path for multi-derive
    ).context("Failed to derive wallet keys")?;

    // Validate generated address
    if !handler.validate_address(&wallet_keys.address) {
        bail!("Generated address failed validation");
    }

    // Check if wallet already exists
    if db.get_wallet_by_address(&wallet_keys.address)?.is_some() {
        return Ok(None); // Already exists, skip
    }

    // Generate explorer URL
    let explorer_url = blockchain.get_explorer_url(&wallet_keys.address);

    // Create wallet record
    let wallet_record = WalletRecord {
        id: None,
        label: Some(format!("{}_{}", args.group_name, blockchain.to_string().to_lowercase())),
        blockchain: blockchain.to_string().to_lowercase(),
        address: wallet_keys.address.clone(),
        address_with_checksum: wallet_keys.address_with_checksum.clone(),
        public_key: Some(wallet_keys.public_key.clone()),
        private_key: wallet_keys.private_key.clone(),
        mnemonic: Some(mnemonic.to_string()),
        passphrase: args.passphrase.clone(),
        derivation_path: wallet_keys.derivation_path.clone(),
        account: Some(args.account.unwrap_or(0)),
        address_index: Some(args.address_index.unwrap_or(0)),
        source_type: "mnemonic".to_string(),
        explorer_url: Some(explorer_url),
        imported_at: Utc::now(),
        notes: None,
        additional_data: wallet_keys.additional_data.clone(),
        secondary_addresses: wallet_keys.secondary_addresses.clone(),
        group_id: Some(group_id), // Link to wallet group
    };

    // Insert into database
    let wallet_id = db.insert_wallet(&wallet_record)
        .context("Failed to save wallet to database")?;

    Ok(Some(wallet_id))
}

