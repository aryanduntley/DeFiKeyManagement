use anyhow::{Result, Context};
use clap::Args;
use chrono::Utc;

use crate::database::{Database, Wallet};
use crate::blockchain::{SupportedBlockchain, get_blockchain_handler};

#[derive(Args)]
pub struct AddWalletArgs {
    #[arg(long, help = "Name of the master account")]
    pub account: String,
    #[arg(long, help = "Name of the wallet group")]
    pub wallet_group: String,
    #[arg(long, help = "Single blockchain to add (e.g., 'bitcoin', 'ethereum', 'solana')")]
    pub blockchain: String,
    #[arg(long, help = "Name/label for the wallet")]
    pub name: String,
    #[arg(long, help = "Account index for derivation (default: use wallet group's account index)")]
    pub account_index: Option<u32>,
    #[arg(long, help = "Starting address index (default: 0)")]
    pub address_index: Option<u32>,
}

pub fn execute(args: AddWalletArgs, db: &Database) -> Result<()> {
    println!("Adding wallet '{}' to wallet group: {}", args.name, args.wallet_group);
    println!("Master account: {}", args.account);
    println!("Blockchain: {}", args.blockchain);

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

    // Parse and validate blockchain
    let blockchain = match SupportedBlockchain::from_str(&args.blockchain.to_lowercase()) {
        Ok(chain) => chain,
        Err(_) => {
            println!("❌ Invalid blockchain: {}", args.blockchain);
            println!("   Supported blockchains: {}", SupportedBlockchain::get_supported_blockchain_names().join(", "));
            return Ok(());
        }
    };

    println!("✓ Blockchain validated: {}", blockchain);

    // Use the wallet group's account index or provided account index
    let account_index = args.account_index.unwrap_or(wallet_group.account_index);
    let address_index = args.address_index.unwrap_or(0);

    // Process single blockchain
    println!("\nProcessing {}...", blockchain);

    match process_blockchain(
        db,
        &master_account.mnemonic,
        master_account.passphrase.as_deref(),
        wallet_group.id.unwrap(),
        &blockchain,
        account_index,
        address_index,
        &args.name,
    ) {
        Ok(wallet_id) => {
            println!("✓ Success (Wallet ID: {})", wallet_id);
            println!("\n🎉 Wallet '{}' created successfully!", args.name);
        }
        Err(e) => {
            println!("❌ Failed: {}", e);
            return Ok(());
        }
    }

    println!("\n💡 Next steps:");
    println!("   1. Create address groups: wallet-backup add-address-group --account \"{}\" --wallet-group \"{}\" --wallet \"{}\" --name \"receiving\"", args.account, args.wallet_group, args.name);
    println!("   2. List wallets: wallet-backup list-wallets --account \"{}\" --wallet-group \"{}\"", args.account, args.wallet_group);
    println!("   3. Show group details: wallet-backup show-wallet-group --account \"{}\" --group \"{}\"", args.account, args.wallet_group);

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
    wallet_name: &str,
) -> Result<i64> {
    // Get blockchain handler
    let handler = get_blockchain_handler(blockchain)?;

    // For base wallets, we derive child private keys (m/account_index)
    // The address_index is not used for base wallets, only for subwallets
    let wallet_keys = handler.derive_from_mnemonic(
        mnemonic,
        passphrase,
        account_index,
        0, // Use 0 for the base wallet derivation
        None, // use default derivation path
    ).context("Failed to derive keys from mnemonic")?;

    // Create BASE WALLET record (address_group_id = None)
    let wallet = Wallet {
        id: None,
        wallet_group_id: Some(wallet_group_id),
        address_group_id: None, // None = base wallet (child private key)
        blockchain: blockchain.to_string(),
        address: wallet_keys.address.clone(),
        address_with_checksum: wallet_keys.address_with_checksum.clone(),
        private_key: wallet_keys.private_key,
        public_key: Some(wallet_keys.public_key),
        derivation_path: Some(wallet_keys.derivation_path),
        label: Some(wallet_name.to_string()),
        source_type: "mnemonic".to_string(),
        explorer_url: Some(blockchain.get_explorer_url(&wallet_keys.address)),
        notes: None,
        created_at: Utc::now(),
        additional_data: wallet_keys.additional_data,
        secondary_addresses: wallet_keys.secondary_addresses,
    };

    // Insert into database
    let wallet_id = db.create_wallet(&wallet)?;

    Ok(wallet_id)
}