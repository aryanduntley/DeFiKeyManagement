use anyhow::{Result, bail, Context};
use chrono::Utc;
use crate::cli::ImportMultiArgs;
use crate::database::{Database, WalletAddress};
use crate::blockchain::{SupportedBlockchain, get_blockchain_handler};
use crate::crypto::bip39_utils::{validate_mnemonic_phrase, normalize_mnemonic};

pub fn execute(args: ImportMultiArgs, db: &Database) -> Result<()> {
    println!("Starting multi-blockchain wallet import...");
    println!("Group: {}", args.group_name);

    // Normalize and validate mnemonic
    let normalized_mnemonic = normalize_mnemonic(&args.mnemonic);
    validate_mnemonic_phrase(&normalized_mnemonic)
        .context("Invalid mnemonic phrase")?;

    // Parse and validate blockchains
    let blockchain_names: Vec<String> = if let Some(chains) = &args.blockchains {
        // User specified specific blockchains
        chains.split(',')
            .map(|s| s.trim().to_lowercase())
            .collect()
    } else {
        // Default to common blockchains for multi-wallet
        vec![
            "bitcoin".to_string(),
            "ethereum".to_string(),
            "solana".to_string(),
            "polygon".to_string(),
            "binance".to_string(),
        ]
    };

    // Validate all requested blockchains are supported
    let supported_blockchains = SupportedBlockchain::validate_blockchains(&blockchain_names)
        .context("Blockchain validation failed")?;

    println!("Blockchains to derive: {}", blockchain_names.join(", "));

    // Create or get wallet group
    let group_id = db.create_or_get_wallet_group(
        &args.group_name,
        args.description.as_deref(),
        &normalized_mnemonic,
        &blockchain_names,
    ).context("Failed to create wallet group")?;

    println!("Wallet group '{}' ready (ID: {})", args.group_name, group_id);

    // Derive wallets for each blockchain
    let mut success_count = 0;
    let mut skipped_count = 0;
    let mut failed_blockchains = Vec::new();

    for blockchain in supported_blockchains {
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
    println!("\n🎊 Multi-blockchain import complete!");
    println!("  Group: {}", args.group_name);
    println!("  ✅ Successfully imported: {}", success_count);
    if skipped_count > 0 {
        println!("  ⚠️  Skipped (already exist): {}", skipped_count);
    }
    if !failed_blockchains.is_empty() {
        println!("  ❌ Failed: {}", failed_blockchains.join(", "));
    }
    println!("  📊 Total wallets in group: {}", success_count + skipped_count);

    if success_count > 0 {
        println!("\nUse 'wallet-backup show-group \"{}\"' to see all wallets in this group.", args.group_name);
    }

    Ok(())
}

fn derive_wallet_for_blockchain(
    blockchain: &SupportedBlockchain,
    mnemonic: &str,
    args: &ImportMultiArgs,
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
        None, // No custom path for multi-import
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

impl std::fmt::Display for SupportedBlockchain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            SupportedBlockchain::Bitcoin => "bitcoin",
            SupportedBlockchain::Ethereum => "ethereum",
            SupportedBlockchain::Solana => "solana",
            SupportedBlockchain::Stellar => "stellar",
            SupportedBlockchain::XRP => "xrp",
            SupportedBlockchain::Cardano => "cardano",
            SupportedBlockchain::Tron => "tron",
            SupportedBlockchain::Cronos => "cronos",
            SupportedBlockchain::Hedera => "hedera",
            SupportedBlockchain::Algorand => "algorand",
            SupportedBlockchain::Cosmos => "cosmos",
            SupportedBlockchain::BinanceBNB => "binance",
            SupportedBlockchain::Litecoin => "litecoin",
            SupportedBlockchain::Polygon => "polygon",
            SupportedBlockchain::Polkadot => "polkadot",
            SupportedBlockchain::Sui => "sui",
            SupportedBlockchain::Optimism => "optimism",
            SupportedBlockchain::IOTA => "iota",
            SupportedBlockchain::XDC => "xdc",
            SupportedBlockchain::TON => "ton",
        };
        write!(f, "{}", name)
    }
}