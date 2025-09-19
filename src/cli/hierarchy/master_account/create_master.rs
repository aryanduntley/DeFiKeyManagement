use anyhow::{Result, Context};
use clap::Args;
use crate::database::Database;
use crate::crypto::bip39_utils::{validate_mnemonic_phrase, normalize_mnemonic, generate_seed_from_mnemonic};

#[derive(Args)]
pub struct CreateMasterArgs {
    #[arg(long, help = "Name for the master account")]
    pub account_name: String,
    #[arg(long, help = "BIP-39 mnemonic phrase (12-24 words)")]
    pub mnemonic: String,
    #[arg(long, help = "Optional passphrase for additional security")]
    pub passphrase: Option<String>,
}

pub fn execute(args: CreateMasterArgs, db: &Database) -> Result<()> {
    println!("Creating account: {}", args.account_name);

    // Validate and normalize mnemonic
    let normalized_mnemonic = normalize_mnemonic(&args.mnemonic);
    validate_mnemonic_phrase(&normalized_mnemonic)
        .context("Invalid mnemonic phrase")?;

    println!("✓ Mnemonic phrase validated");

    // Generate seed from mnemonic and passphrase
    let seed = generate_seed_from_mnemonic(&normalized_mnemonic, args.passphrase.as_deref())
        .context("Failed to generate seed from mnemonic")?;

    // For master private key, we'll use the first 32 bytes of the seed as hex
    // This is a simplified approach - in production we might want proper BIP32 master key derivation
    let master_private_key = hex::encode(&seed[0..32]);

    println!("✓ Master private key derived");

    // Check if account already exists
    if let Some(_existing) = db.get_master_account_by_name(&args.account_name)? {
        println!("❌ Account '{}' already exists.", args.account_name);
        println!("   Use a different account name or delete the existing account first.");
        return Ok(());
    }

    // Create master account in database
    let master_id = db.create_master_account(
        &args.account_name,
        &normalized_mnemonic,
        &master_private_key,
        args.passphrase.as_deref(),
    ).context("Failed to create master account in database")?;

    // Success message
    println!("\n🎉 Account created successfully!");
    println!("   Account Name: {}", args.account_name);
    println!("   Account ID: {}", master_id);
    println!("   Next Account Index: 0 (ready for wallet groups)");

    if args.passphrase.is_some() {
        println!("   Passphrase: ✓ (protected)");
    }

    println!("\n💡 Next steps:");
    println!("   1. Create a wallet group: wallet-backup create-wallet-group --account \"{}\" --name \"PersonalWallet\"", args.account_name);
    println!("   2. List accounts: wallet-backup list-accounts");

    Ok(())
}