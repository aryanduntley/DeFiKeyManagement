use anyhow::{Result, bail, Context};
use chrono::Utc;
use crate::cli::ImportArgs;
use crate::database::{Database, WalletAddress};
use crate::blockchain::{SupportedBlockchain, get_blockchain_handler};
use crate::crypto::bip39_utils::{validate_mnemonic_phrase, normalize_mnemonic};

pub fn execute(args: ImportArgs, db: &Database) -> Result<()> {
    // Validate input - must have either mnemonic or private key
    match (&args.mnemonic, &args.private_key) {
        (None, None) => bail!("Must provide either --mnemonic or --private-key"),
        (Some(_), Some(_)) => bail!("Cannot provide both --mnemonic and --private-key"),
        _ => {} // Valid: exactly one is provided
    }

    println!("Importing wallet for blockchain: {}", args.blockchain);

    // Parse blockchain
    let blockchain = SupportedBlockchain::from_str(&args.blockchain)
        .context("Unsupported blockchain")?;
    
    // Get blockchain handler
    let handler = get_blockchain_handler(&blockchain)
        .context("Failed to get blockchain handler")?;

    // Derive wallet keys
    let wallet_keys = if let Some(mnemonic) = &args.mnemonic {
        // Import from mnemonic
        let normalized_mnemonic = normalize_mnemonic(mnemonic);
        validate_mnemonic_phrase(&normalized_mnemonic)
            .context("Invalid mnemonic phrase")?;

        println!("Deriving keys from mnemonic...");
        handler.derive_from_mnemonic(
            &normalized_mnemonic,
            args.passphrase.as_deref(),
            0, // Default account
            0, // Default address index
            args.custom_path.as_deref(),
        )?
    } else if let Some(private_key) = &args.private_key {
        // Import from private key
        println!("Deriving keys from private key...");
        handler.derive_from_private_key(private_key)?
    } else {
        unreachable!("Validation above ensures one is provided");
    };

    // Validate generated address
    if !handler.validate_address(&wallet_keys.address) {
        bail!("Generated address failed validation");
    }

    // Check if wallet already exists
    if let Some(existing) = db.get_wallet_address_by_address(&wallet_keys.address)? {
        println!("Wallet already exists:");
        println!("  Address: {}", existing.address);
        println!("  Label: {}", existing.label.as_deref().unwrap_or("(no label)"));
        return Ok(());
    }

    // Generate explorer URL
    let explorer_url = blockchain.get_explorer_url(&wallet_keys.address);

    // Create wallet address record
    let wallet_record = WalletAddress {
        id: None,
        wallet_group_id: None, // Will be NULL for orphaned addresses
        address_group_id: None, // Will be NULL for orphaned addresses
        blockchain: args.blockchain.clone(),
        address: wallet_keys.address.clone(),
        address_with_checksum: wallet_keys.address_with_checksum.clone(),
        private_key: wallet_keys.private_key.clone(),
        public_key: Some(wallet_keys.public_key.clone()),
        derivation_path: if args.mnemonic.is_some() { Some(wallet_keys.derivation_path.clone()) } else { None },
        address_index: if args.mnemonic.is_some() { Some(0) } else { None },
        label: args.label.clone(),
        source_type: if args.mnemonic.is_some() { "mnemonic" } else { "private_key" }.to_string(),
        explorer_url: Some(explorer_url),
        imported_at: Utc::now(),
        notes: None,
        created_at: Utc::now(),
        additional_data: wallet_keys.additional_data.clone(),
        secondary_addresses: wallet_keys.secondary_addresses.clone(),
    };

    // Insert into database (as orphaned address)
    let wallet_id = db.create_orphaned_wallet_address(&wallet_record)
        .context("Failed to save wallet to database")?;

    // Success message
    println!("\n✓ Wallet imported successfully!");
    println!("  ID: {}", wallet_id);
    println!("  Label: {}", args.label.as_deref().unwrap_or("(no label)"));
    println!("  Blockchain: {}", args.blockchain);
    println!("  Address: {}", wallet_keys.address);

    // Show checksummed address if available
    if let Some(checksummed) = &wallet_keys.address_with_checksum {
        println!("  Address (with checksum): {}", checksummed);
    }

    // Show secondary addresses if any
    if !wallet_keys.secondary_addresses.is_empty() {
        println!("  Secondary addresses:");
        for (addr_type, addr) in &wallet_keys.secondary_addresses {
            println!("    {}: {}", addr_type.to_uppercase(), addr);
        }
    }

    // Show additional data if any
    if !wallet_keys.additional_data.is_empty() {
        println!("  Additional data:");
        for (key, value) in &wallet_keys.additional_data {
            println!("    {}: {}", key, value);
        }
    }

    println!("  Derivation Path: {}", wallet_keys.derivation_path);
    println!("  Explorer: {}", wallet_record.explorer_url.as_deref().unwrap_or("N/A"));

    Ok(())
}