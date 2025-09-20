use anyhow::{Result, Context};
use clap::Args;
use chrono::Utc;

use crate::database::{Database, Wallet};
use crate::blockchain::{SupportedBlockchain, get_blockchain_handler};

#[derive(Args)]
pub struct AddSubwalletArgs {
    #[arg(long, help = "Name of the master account")]
    pub account: String,
    #[arg(long, help = "Name of the wallet group")]
    pub wallet_group: String,
    #[arg(long, help = "Name of the base wallet")]
    pub wallet: String,
    #[arg(long, help = "Name of the address group")]
    pub address_group: String,
    #[arg(long, help = "Name/label for the subwallet")]
    pub name: String,
    #[arg(long, help = "Address index for derivation (default: auto-increment)")]
    pub address_index: Option<u32>,
}

pub fn execute(args: AddSubwalletArgs, db: &Database) -> Result<()> {
    println!("💰 Adding subwallet '{}' to address group", args.name);
    println!("Account: {}", args.account);
    println!("Wallet Group: {}", args.wallet_group);
    println!("Base Wallet: {}", args.wallet);
    println!("Address Group: {}", args.address_group);

    // Get master account by name
    let master_account = match db.get_master_account_by_name(&args.account)? {
        Some(account) => account,
        None => {
            println!("\n❌ Master account '{}' not found.", args.account);
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

    // Get the base wallet by name within this wallet group
    let base_wallet = match db.get_wallet_by_name_in_group(wallet_group.id.unwrap(), &args.wallet)? {
        Some(wallet) => wallet,
        None => {
            println!("❌ Base wallet '{}' not found in wallet group '{}'.", args.wallet, args.wallet_group);
            println!("   Use 'wallet-backup list-wallets --account \"{}\" --wallet-group \"{}\"' to see available wallets.", args.account, args.wallet_group);
            return Ok(());
        }
    };

    println!("✓ Base wallet found (ID: {}, Blockchain: {})", base_wallet.id.unwrap_or(-1), base_wallet.blockchain);

    // Get the address group by name for this wallet
    let address_group = match db.get_address_group_by_name_for_wallet(base_wallet.id.unwrap(), &args.address_group)? {
        Some(group) => group,
        None => {
            println!("❌ Address group '{}' not found for wallet '{}'.", args.address_group, args.wallet);
            println!("   Use 'wallet-backup list-address-groups --account \"{}\" --wallet-group \"{}\" --wallet \"{}\"' to see available address groups.", args.account, args.wallet_group, args.wallet);
            return Ok(());
        }
    };

    println!("✓ Address group found (ID: {}, Index: {})", address_group.id.unwrap_or(-1), address_group.address_group_index);

    // Parse blockchain from base wallet
    let blockchain = match SupportedBlockchain::from_str(&base_wallet.blockchain.to_lowercase()) {
        Ok(chain) => chain,
        Err(_) => {
            println!("❌ Invalid blockchain: {}", base_wallet.blockchain);
            return Ok(());
        }
    };

    println!("✓ Blockchain validated: {}", blockchain);

    // Determine address index (auto-increment if not provided)
    let address_index = if let Some(idx) = args.address_index {
        idx
    } else {
        // Get highest existing address index in this address group and increment
        let existing_subwallets = db.get_wallets_by_address_group(address_group.id.unwrap())?;
        let max_index = existing_subwallets.iter()
            .filter_map(|w| w.derivation_path.as_ref())
            .filter_map(|path| {
                // Extract address index from path like "m/44'/0'/0'/0/3"
                path.split('/').last()?.parse::<u32>().ok()
            })
            .max()
            .unwrap_or(0);

        max_index + 1
    };

    println!("✓ Using address index: {}", address_index);

    // Derive the subwallet keys (grandchild private key)
    // Derivation path: m/44'/coin_type'/account_index'/wallet_index/address_index
    let handler = get_blockchain_handler(&blockchain)?;

    let subwallet_keys = handler.derive_from_mnemonic(
        &master_account.mnemonic,
        master_account.passphrase.as_deref(),
        wallet_group.account_index,
        address_index,
        None, // use default derivation path
    ).context("Failed to derive subwallet keys from mnemonic")?;

    println!("✓ Subwallet keys derived successfully");
    println!("   Derivation Path: {}", subwallet_keys.derivation_path);
    println!("   Address: {}", subwallet_keys.address);

    // Create SUBWALLET record (address_group_id = Some(id))
    let subwallet = Wallet {
        id: None,
        wallet_group_id: Some(wallet_group.id.unwrap()),
        address_group_id: Some(address_group.id.unwrap()), // This makes it a subwallet
        blockchain: blockchain.to_string(),
        address: subwallet_keys.address.clone(),
        address_with_checksum: subwallet_keys.address_with_checksum.clone(),
        private_key: subwallet_keys.private_key,
        public_key: Some(subwallet_keys.public_key),
        derivation_path: Some(subwallet_keys.derivation_path),
        label: Some(args.name.clone()),
        source_type: "mnemonic".to_string(),
        explorer_url: Some(blockchain.get_explorer_url(&subwallet_keys.address)),
        notes: None,
        created_at: Utc::now(),
        additional_data: subwallet_keys.additional_data,
        secondary_addresses: subwallet_keys.secondary_addresses,
    };

    // Insert into database
    let subwallet_id = db.create_wallet(&subwallet)?;

    println!("\n🎉 Subwallet '{}' created successfully!", args.name);
    println!("   Subwallet ID: {}", subwallet_id);
    println!("   Address: {}", subwallet.address);
    println!("   Blockchain: {}", blockchain);
    println!("   Address Group: {}", args.address_group);

    println!("\n💡 Next steps:");
    println!("   • View all subwallets: wallet-backup list-subwallets --account \"{}\" --wallet-group \"{}\" --wallet \"{}\" --address-group \"{}\"", args.account, args.wallet_group, args.wallet, args.address_group);
    println!("   • Add more subwallets: wallet-backup add-subwallet --account \"{}\" --wallet-group \"{}\" --wallet \"{}\" --address-group \"{}\" --name \"<name>\"", args.account, args.wallet_group, args.wallet, args.address_group);
    println!("   • Show wallet details: wallet-backup show-wallet --address \"{}\" --include-sensitive", subwallet.address);

    Ok(())
}