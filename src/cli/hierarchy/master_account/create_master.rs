use anyhow::{Result, Context};
use clap::Args;
use crate::database::Database;
use crate::crypto::bip39_utils::{validate_mnemonic_phrase, normalize_mnemonic, generate_seed_from_mnemonic};

#[derive(Args)]
pub struct CreateMasterArgs {
    #[arg(long, help = "Name for the account")]
    pub name: String,
    #[arg(long, help = "BIP-39 mnemonic phrase (12-24 words)", conflicts_with = "master_private")]
    pub mnemonic: Option<String>,
    #[arg(long, help = "Master private key (alternative to mnemonic)", conflicts_with = "mnemonic")]
    pub master_private: Option<String>,
    #[arg(long, help = "Optional passphrase for additional security")]
    pub passphrase: Option<String>,
}

pub fn execute(args: CreateMasterArgs, db: &Database) -> Result<()> {
    println!("Creating account: {}", args.name);

    // Validate input - must have either mnemonic or master_private
    let (normalized_mnemonic, master_private_key, source_type) = match (&args.mnemonic, &args.master_private) {
        (Some(mnemonic), None) => {
            // Using mnemonic
            let normalized_mnemonic = normalize_mnemonic(mnemonic);
            validate_mnemonic_phrase(&normalized_mnemonic)
                .context("Invalid mnemonic phrase")?;
            println!("✓ Mnemonic phrase validated");

            // Generate seed from mnemonic and passphrase
            let seed = generate_seed_from_mnemonic(&normalized_mnemonic, args.passphrase.as_deref())
                .context("Failed to generate seed from mnemonic")?;

            // For master private key, use the first 32 bytes of the seed as hex
            let master_private = hex::encode(&seed[0..32]);
            (normalized_mnemonic, master_private, "mnemonic")
        },
        (None, Some(master_private)) => {
            // Using master private key
            println!("✓ Using provided master private key");
            // For mnemonic storage, we'll store a placeholder since we don't have the original mnemonic
            ("[MASTER_PRIVATE_KEY_SOURCE]".to_string(), master_private.clone(), "master_private_key")
        },
        (None, None) => {
            return Err(anyhow::anyhow!("Must provide either --mnemonic or --master-private"));
        },
        (Some(_), Some(_)) => {
            return Err(anyhow::anyhow!("Cannot provide both --mnemonic and --master-private"));
        }
    };

    println!("✓ Master private key derived");

    // Check if account already exists
    if let Some(_existing) = db.get_master_account_by_name(&args.name)? {
        println!("❌ Account '{}' already exists.", args.name);
        println!("   Use a different account name or delete the existing account first.");
        return Ok(());
    }

    // Create master account in database
    let master_id = db.create_master_account(
        &args.name,
        &normalized_mnemonic,
        &master_private_key,
        args.passphrase.as_deref(),
    ).context("Failed to create master account in database")?;

    // Success message
    println!("\n🎉 Account created successfully!");
    println!("   Account Name: {}", args.name);
    println!("   Account ID: {}", master_id);
    println!("   Next Account Index: 0 (ready for wallet groups)");

    if args.passphrase.is_some() {
        println!("   Passphrase: ✓ (protected)");
    }

    println!("\n💡 Next steps:");
    println!("   1. Create a wallet group: wallet-backup add-wallet-group --account \"{}\" --name \"PersonalWallet\"", args.name);
    println!("   2. List accounts: wallet-backup list-accounts");

    Ok(())
}