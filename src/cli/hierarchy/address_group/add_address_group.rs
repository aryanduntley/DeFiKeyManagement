use anyhow::Result;
use clap::Args;
use crate::database::Database;

#[derive(Args)]
pub struct AddAddressGroupArgs {
    #[arg(long, help = "Name of the master account")]
    pub account: String,
    #[arg(long, help = "Name of the wallet group")]
    pub wallet_group: String,
    #[arg(long, help = "Name of the base wallet")]
    pub wallet: String,
    #[arg(long, help = "Name for the address group")]
    pub name: String,
    #[arg(long, help = "Optional description for the address group")]
    pub description: Option<String>,
}

pub fn execute(args: AddAddressGroupArgs, db: &Database) -> Result<()> {
    println!("Creating address group: {}", args.name);
    println!("Master account: {}", args.account);
    println!("Wallet group: {}", args.wallet_group);
    println!("Base wallet: {}", args.wallet);

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

    println!("✓ Wallet group found (ID: {})", wallet_group.id.unwrap_or(-1));

    // Get base wallet by name within the wallet group
    let base_wallet = match db.get_wallet_by_name_in_group(wallet_group.id.unwrap(), &args.wallet)? {
        Some(wallet) => wallet,
        None => {
            println!("❌ Base wallet '{}' not found in wallet group '{}'.", args.wallet, args.wallet_group);
            println!("   Use 'wallet-backup list-wallets --account \"{}\" --wallet-group \"{}\"' to see available wallets.", args.account, args.wallet_group);
            return Ok(());
        }
    };

    println!("✓ Base wallet found (ID: {}, Blockchain: {})", base_wallet.id.unwrap_or(-1), base_wallet.blockchain);

    // Check if address group with this name already exists for this wallet
    let existing_groups = db.list_address_groups_for_wallet(base_wallet.id.unwrap())?;
    for group in &existing_groups {
        if group.name == args.name {
            println!("❌ Address group '{}' already exists for wallet '{}' ({}).", args.name, args.wallet, base_wallet.blockchain);
            println!("   Use a different name or manage the existing group.");
            return Ok(());
        }
    }

    // Create address group for the specific wallet
    let address_group_id = db.create_address_group(
        wallet_group.id.unwrap(),
        base_wallet.id.unwrap(),
        &base_wallet.blockchain,
        &args.name,
    )?;

    println!();
    println!("🎉 Address group created successfully!");
    println!("   Group Name: {}", args.name);
    println!("   Group ID: {}", address_group_id);
    println!("   Base Wallet: {} ({})", args.wallet, base_wallet.blockchain);
    println!("   Wallet Group: {}", args.wallet_group);
    println!("   Master Account: {}", args.account);
    if let Some(desc) = &args.description {
        println!("   Description: {}", desc);
    }

    println!();
    println!("💡 Next steps:");
    println!("   1. Add subwallets: wallet-backup add-subwallet --account \"{}\" --wallet-group \"{}\" --wallet \"{}\" --address-group \"{}\" --name \"addr1\"", args.account, args.wallet_group, args.wallet, args.name);
    println!("   2. List address groups: wallet-backup list-address-groups --account \"{}\" --wallet-group \"{}\" --wallet \"{}\"", args.account, args.wallet_group, args.wallet);
    println!("   3. Show wallet details: wallet-backup show-wallet-group --account \"{}\" --group \"{}\"", args.account, args.wallet_group);

    Ok(())
}