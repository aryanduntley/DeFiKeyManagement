use anyhow::{Result, Context};
use clap::Args;
use chrono::Utc;

use crate::database::{Database, WalletAddress};
use crate::blockchain::{SupportedBlockchain, get_blockchain_handler};

#[derive(Args)]
pub struct AddBlockchainArgs {
    #[arg(long, help = "Name of the master account")]
    pub account: String,
    #[arg(long, help = "Name of the wallet group")]
    pub wallet_group: String,
    #[arg(long, help = "Comma-separated list of blockchains to add (e.g., 'bitcoin,ethereum,solana')")]
    pub blockchains: String,
    #[arg(long, help = "Account index for derivation (default: use wallet group's account index)")]
    pub account_index: Option<u32>,
    #[arg(long, help = "Starting address index (default: 0)")]
    pub address_index: Option<u32>,
}

pub fn execute(args: AddBlockchainArgs, db: &Database) -> Result<()> {
    println!("Adding blockchains to wallet group: {}", args.wallet_group);
    println!("Master account: {}", args.account);
    println!("Blockchains: {}", args.blockchains);

    // Get master account by name
    let master_account = match db.get_master_account_by_name(&args.account)? {
        Some(account) => account,
        None => {
            println!("❌ Master account '{}' not found.", args.account);
            println!("   Use 'wallet-backup list-accounts' to see available accounts.");
            return Ok(());
        }
    };

    println!("✓ Master account found (ID: {})", master_account.id.unwrap_or(-1));

    // Get wallet group by name
    let wallet_group = match db.get_wallet_group_by_name(master_account.id.unwrap(), &args.wallet_group)? {
        Some(group) => group,
        None => {
            println!("❌ Wallet group '{}' not found in account '{}'.", args.wallet_group, args.account);
            println!("   Use 'wallet-backup list-wallet-groups --account \"{}\"' to see available groups.", args.account);
            return Ok(());
        }
    };

    println!("✓ Wallet group found (ID: {}, Account Index: {})", wallet_group.id.unwrap_or(-1), wallet_group.account_index);

    // Parse and validate blockchains
    let blockchain_names: Vec<String> = args.blockchains
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .collect();

    let blockchains = match SupportedBlockchain::validate_blockchains(&blockchain_names) {
        Ok(chains) => chains,
        Err(e) => {
            println!("❌ Invalid blockchain(s): {}", e);
            println!("   Supported blockchains: {}", SupportedBlockchain::get_supported_blockchain_names().join(", "));
            return Ok(());
        }
    };

    println!("✓ All blockchains validated: {}", blockchains.len());

    // Use the wallet group's account index or provided account index
    let account_index = args.account_index.unwrap_or(wallet_group.account_index);
    let address_index = args.address_index.unwrap_or(0);

    let mut success_count = 0;
    let mut failed_blockchains = Vec::new();

    // Process each blockchain
    for blockchain in blockchains {
        println!("\nProcessing {}...", blockchain);

        match process_blockchain(
            db,
            &master_account.mnemonic,
            master_account.passphrase.as_deref(),
            wallet_group.id.unwrap(),
            &blockchain,
            account_index,
            address_index,
            &args.wallet_group,
        ) {
            Ok(address_id) => {
                println!("  ✓ Success (Address ID: {})", address_id);
                success_count += 1;
            }
            Err(e) => {
                println!("  ❌ Failed: {}", e);
                failed_blockchains.push(blockchain.to_string());
            }
        }
    }

    // Summary
    println!("\n🎊 Add blockchain operation complete!");
    println!("  ✅ Successfully added: {}", success_count);

    if !failed_blockchains.is_empty() {
        println!("  ❌ Failed: {} ({})", failed_blockchains.len(), failed_blockchains.join(", "));
    }

    println!("\n💡 Next steps:");
    println!("   1. List wallet groups: wallet-backup list-wallet-groups --account \"{}\"", args.account);
    println!("   2. Show group details: wallet-backup show-wallet-group --account \"{}\" --group \"{}\"", args.account, args.wallet_group);

    Ok(())
}

fn process_blockchain(
    db: &Database,
    mnemonic: &str,
    passphrase: Option<&str>,
    wallet_group_id: i64,
    blockchain: &SupportedBlockchain,
    account_index: u32,
    address_index: u32,
    group_name: &str,
) -> Result<i64> {
    // Create or get default address group for this blockchain
    let address_group_id = db.get_or_create_default_address_group(wallet_group_id, &blockchain.to_string())?;

    // Get blockchain handler
    let handler = get_blockchain_handler(blockchain)?;

    // Derive wallet keys from mnemonic
    let wallet_keys = handler.derive_from_mnemonic(
        mnemonic,
        passphrase,
        account_index,
        address_index,
        None, // use default derivation path
    ).context("Failed to derive keys from mnemonic")?;

    // Generate auto label
    let label = format!("{}_{}_{}", group_name, blockchain.to_string().to_lowercase(), address_index);

    // Create WalletAddress record
    let wallet_address = WalletAddress {
        id: None,
        wallet_group_id: Some(wallet_group_id),
        address_group_id: Some(address_group_id),
        blockchain: blockchain.to_string(),
        address: wallet_keys.address.clone(),
        address_with_checksum: wallet_keys.address_with_checksum.clone(),
        private_key: wallet_keys.private_key,
        public_key: Some(wallet_keys.public_key),
        derivation_path: Some(wallet_keys.derivation_path),
        address_index: Some(address_index),
        label: Some(label),
        source_type: "mnemonic".to_string(),
        explorer_url: Some(blockchain.get_explorer_url(&wallet_keys.address)),
        notes: None,
        created_at: Utc::now(),
        additional_data: wallet_keys.additional_data,
        secondary_addresses: wallet_keys.secondary_addresses,
    };

    // Insert into database
    let address_id = db.create_wallet_address(&wallet_address)?;

    Ok(address_id)
}