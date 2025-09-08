use anyhow::{Result, bail, Context};
use chrono::Utc;
use crate::cli::ImportArgs;
use crate::database::{Database, WalletRecord};
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
    if let Some(existing) = db.get_wallet_by_address(&wallet_keys.address)? {
        println!("Wallet already exists:");
        println!("  Address: {}", existing.address);
        println!("  Label: {}", existing.label.as_deref().unwrap_or("(no label)"));
        return Ok(());
    }

    // Generate explorer URL
    let explorer_url = blockchain.get_explorer_url(&wallet_keys.address);

    // Create wallet record
    let wallet_record = WalletRecord {
        id: None,
        label: args.label.clone(),
        blockchain: args.blockchain.clone(),
        address: wallet_keys.address.clone(),
        public_key: Some(wallet_keys.public_key.clone()),
        private_key: wallet_keys.private_key.clone(),
        mnemonic: args.mnemonic.as_ref().map(|m| normalize_mnemonic(m)),
        passphrase: args.passphrase.clone(),
        derivation_path: wallet_keys.derivation_path.clone(),
        account: Some(0), // Default account for mnemonic imports
        address_index: Some(0), // Default address index
        source_type: if args.mnemonic.is_some() { "mnemonic" } else { "private_key" }.to_string(),
        explorer_url: Some(explorer_url),
        imported_at: Utc::now(),
        notes: None,
    };

    // Insert into database
    let wallet_id = db.insert_wallet(&wallet_record)
        .context("Failed to save wallet to database")?;

    // Success message
    println!("\n✓ Wallet imported successfully!");
    println!("  ID: {}", wallet_id);
    println!("  Label: {}", args.label.as_deref().unwrap_or("(no label)"));
    println!("  Blockchain: {}", args.blockchain);
    println!("  Address: {}", wallet_keys.address);
    println!("  Derivation Path: {}", wallet_keys.derivation_path);
    println!("  Explorer: {}", wallet_record.explorer_url.as_deref().unwrap_or("N/A"));

    Ok(())
}