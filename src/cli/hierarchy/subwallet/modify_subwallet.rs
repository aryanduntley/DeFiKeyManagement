use anyhow::{Result, Context};
use clap::Args;
use crate::database::Database;
use std::io::{self, Write};

#[derive(Args)]
pub struct ModifySubwalletArgs {
    #[arg(long, help = "Name of the master account")]
    pub account: String,
    #[arg(long, help = "Name of the wallet group")]
    pub wallet_group: String,
    #[arg(long, help = "Name or address of the wallet")]
    pub wallet: String,
    #[arg(long, help = "Name of the address group")]
    pub address_group: String,
    #[arg(long, help = "Name of the subwallet to modify", conflicts_with = "address")]
    pub subwallet: Option<String>,
    #[arg(long, help = "Address of the subwallet to modify", conflicts_with = "subwallet")]
    pub address: Option<String>,

    // Modification options
    #[arg(long, help = "Set or update subwallet label")]
    pub label: Option<String>,
    #[arg(long, help = "Set or update notes")]
    pub notes: Option<String>,
    #[arg(long, help = "Add additional data as key=value pairs", value_parser = parse_key_val)]
    pub add_data: Vec<(String, String)>,
    #[arg(long, help = "Remove additional data by key")]
    pub remove_data: Vec<String>,
    #[arg(long, help = "Add secondary address as type=address pairs", value_parser = parse_key_val)]
    pub add_secondary: Vec<(String, String)>,
    #[arg(long, help = "Remove secondary address by type")]
    pub remove_secondary: Vec<String>,
    #[arg(long, help = "Clear all additional data")]
    pub clear_data: bool,
    #[arg(long, help = "Clear all secondary addresses")]
    pub clear_secondary: bool,
}

fn parse_key_val(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid key=value format: {}", s));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

pub fn execute(args: ModifySubwalletArgs, db: &Database) -> Result<()> {
    println!("🔧 Modifying subwallet");
    println!("Account: {}", args.account);
    println!("Wallet Group: {}", args.wallet_group);
    println!("Wallet: {}", args.wallet);
    println!("Address Group: {}", args.address_group);

    // Validate that either subwallet or address is provided
    let identifier = if let Some(ref name) = args.subwallet {
        println!("Subwallet Name: {}", name);
        name.clone()
    } else if let Some(ref addr) = args.address {
        println!("Subwallet Address: {}", addr);
        addr.clone()
    } else {
        println!("❌ Either --subwallet-name or --address must be provided.");
        return Ok(());
    };

    // Get master account by name
    let master_account = match db.get_master_account_by_name(&args.account)? {
        Some(account) => account,
        None => {
            println!("\n❌ Master account '{}' not found.", args.account);
            return Ok(());
        }
    };

    // Get wallet group by name
    let wallet_group = match db.get_wallet_group_by_name(master_account.id.unwrap(), &args.wallet_group)? {
        Some(group) => group,
        None => {
            println!("❌ Wallet group '{}' not found in account '{}'.", args.wallet_group, args.account);
            return Ok(());
        }
    };

    // Get all wallets in the wallet group to find the base wallet
    let wallets = db.get_wallets_by_wallet_group(wallet_group.id.unwrap())
        .context("Failed to get wallets")?;

    // Find the base wallet by name or address
    let base_wallet = wallets.into_iter().find(|w| {
        (if let Some(label) = &w.label {
            label == &args.wallet
        } else {
            false
        }) || w.address == args.wallet
    });

    let wallet = match base_wallet {
        Some(w) => w,
        None => {
            println!("❌ Wallet '{}' not found in wallet group '{}'.", args.wallet, args.wallet_group);
            return Ok(());
        }
    };

    // Get address group by name within the wallet
    let address_group = match db.get_address_group_by_name_for_wallet(wallet.id.unwrap(), &args.address_group)? {
        Some(group) => group,
        None => {
            println!("❌ Address group '{}' not found in wallet '{}'.", args.address_group, args.wallet);
            return Ok(());
        }
    };

    // Get all subwallets in the address group
    let subwallets = db.get_wallets_by_address_group(address_group.id.unwrap())
        .context("Failed to get subwallets")?;

    // Find the target subwallet by name or address
    let target_subwallet = if args.subwallet.is_some() {
        // Find by name
        subwallets.into_iter().find(|s| {
            s.label.as_ref().map_or(false, |label| label == &identifier)
        })
    } else {
        // Find by address
        subwallets.into_iter().find(|s| s.address == identifier)
    };

    let mut subwallet = match target_subwallet {
        Some(s) => s,
        None => {
            if args.subwallet.is_some() {
                println!("❌ Subwallet with name '{}' not found in address group '{}'.", identifier, args.address_group);
            } else {
                println!("❌ Subwallet with address '{}' not found in address group '{}'.", identifier, args.address_group);
            }
            return Ok(());
        }
    };

    let mut changes_made = false;

    // Check if any direct modifications were provided via flags
    let has_direct_modifications = args.label.is_some() || args.notes.is_some() ||
        !args.add_data.is_empty() || !args.remove_data.is_empty() ||
        !args.add_secondary.is_empty() || !args.remove_secondary.is_empty() ||
        args.clear_data || args.clear_secondary;

    if has_direct_modifications {
        // Show planned changes for verification
        println!("\n📋 Planned Changes:");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        if let Some(ref new_label) = args.label {
            let old_label = subwallet.label.as_deref().unwrap_or("(none)");
            println!("  Label: '{}' → '{}'", old_label, new_label);
        }

        if let Some(ref new_notes) = args.notes {
            let old_notes = subwallet.notes.as_deref().unwrap_or("(none)");
            println!("  Notes: '{}' → '{}'", old_notes, new_notes);
        }

        for (key, value) in &args.add_data {
            if let Some(existing) = subwallet.additional_data.get(key) {
                println!("  Additional Data '{}': '{}' → '{}' (update)", key, existing, value);
            } else {
                println!("  Additional Data '{}': (none) → '{}' (add)", key, value);
            }
        }

        for key in &args.remove_data {
            if let Some(existing) = subwallet.additional_data.get(key) {
                println!("  Additional Data '{}': '{}' → (removed)", key, existing);
            } else {
                println!("  Additional Data '{}': (not found - no change)", key);
            }
        }

        for (addr_type, address) in &args.add_secondary {
            if let Some(existing) = subwallet.secondary_addresses.get(addr_type) {
                println!("  Secondary Address '{}': '{}' → '{}' (update)", addr_type, existing, address);
            } else {
                println!("  Secondary Address '{}': (none) → '{}' (add)", addr_type, address);
            }
        }

        for addr_type in &args.remove_secondary {
            if let Some(existing) = subwallet.secondary_addresses.get(addr_type) {
                println!("  Secondary Address '{}': '{}' → (removed)", addr_type, existing);
            } else {
                println!("  Secondary Address '{}': (not found - no change)", addr_type);
            }
        }

        if args.clear_data {
            let count = subwallet.additional_data.len();
            if count > 0 {
                println!("  Additional Data: Clear all {} entries", count);
                for key in subwallet.additional_data.keys() {
                    println!("    - '{}'", key);
                }
            } else {
                println!("  Additional Data: Clear all (already empty)");
            }
        }

        if args.clear_secondary {
            let count = subwallet.secondary_addresses.len();
            if count > 0 {
                println!("  Secondary Addresses: Clear all {} addresses", count);
                for addr_type in subwallet.secondary_addresses.keys() {
                    println!("    - '{}'", addr_type);
                }
            } else {
                println!("  Secondary Addresses: Clear all (already empty)");
            }
        }

        // Ask for confirmation
        print!("\nApply these changes? (Y/n): ");
        io::stdout().flush()?;
        let mut confirm_input = String::new();
        io::stdin().read_line(&mut confirm_input)?;
        let confirmation = confirm_input.trim().to_lowercase();

        if confirmation == "n" || confirmation == "no" {
            println!("❌ Changes cancelled.");
            return Ok(());
        }

        println!("✓ Changes confirmed. Applying modifications...\n");
    }

    // Apply modifications
    if let Some(new_label) = args.label {
        let old_label = subwallet.label.clone().unwrap_or("(none)".to_string());
        subwallet.label = Some(new_label.clone());
        println!("✓ Label updated: '{}' → '{}'", old_label, new_label);
        changes_made = true;
    }

    if let Some(new_notes) = args.notes {
        let old_notes = subwallet.notes.clone().unwrap_or("(none)".to_string());
        subwallet.notes = Some(new_notes.clone());
        println!("✓ Notes updated: '{}' → '{}'", old_notes, new_notes);
        changes_made = true;
    }

    // Handle additional data modifications
    if args.clear_data {
        let count = subwallet.additional_data.len();
        subwallet.additional_data.clear();
        println!("✓ Cleared {} additional data entries", count);
        changes_made = true;
    }

    for key in &args.remove_data {
        if subwallet.additional_data.remove(key).is_some() {
            println!("✓ Removed additional data: '{}'", key);
            changes_made = true;
        } else {
            println!("⚠️  Additional data key '{}' not found", key);
        }
    }

    for (key, value) in &args.add_data {
        let old_value = subwallet.additional_data.insert(key.clone(), value.clone());
        if let Some(old) = old_value {
            println!("✓ Updated additional data '{}': '{}' → '{}'", key, old, value);
        } else {
            println!("✓ Added additional data '{}': '{}'", key, value);
        }
        changes_made = true;
    }

    // Handle secondary addresses modifications
    if args.clear_secondary {
        let count = subwallet.secondary_addresses.len();
        subwallet.secondary_addresses.clear();
        println!("✓ Cleared {} secondary addresses", count);
        changes_made = true;
    }

    for addr_type in &args.remove_secondary {
        if subwallet.secondary_addresses.remove(addr_type).is_some() {
            println!("✓ Removed secondary address type: '{}'", addr_type);
            changes_made = true;
        } else {
            println!("⚠️  Secondary address type '{}' not found", addr_type);
        }
    }

    for (addr_type, address) in &args.add_secondary {
        let old_address = subwallet.secondary_addresses.insert(addr_type.clone(), address.clone());
        if let Some(old) = old_address {
            println!("✓ Updated secondary address '{}': '{}' → '{}'", addr_type, old, address);
        } else {
            println!("✓ Added secondary address '{}': '{}'", addr_type, address);
        }
        changes_made = true;
    }

    // If no direct modifications provided, enter interactive mode
    if !changes_made {
        println!("\n🔧 Interactive Modification Mode");
        println!("Current subwallet: {}", subwallet.label.as_deref().unwrap_or(&subwallet.address));

        loop {
            println!("\nWhat would you like to modify?");
            println!("1. Label (current: {})", subwallet.label.as_deref().unwrap_or("(none)"));
            println!("2. Notes (current: {})", subwallet.notes.as_deref().unwrap_or("(none)"));
            println!("3. Add additional data");
            println!("4. Remove additional data");
            println!("5. Add secondary address");
            println!("6. Remove secondary address");
            println!("7. Clear all additional data ({} entries)", subwallet.additional_data.len());
            println!("8. Clear all secondary addresses ({} addresses)", subwallet.secondary_addresses.len());
            println!("9. Exit");

            print!("Choose option (1-9): ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let choice = input.trim();

            match choice {
                "1" => {
                    print!("Enter new label (current: {}): ", subwallet.label.as_deref().unwrap_or("(none)"));
                    io::stdout().flush()?;
                    let mut label_input = String::new();
                    io::stdin().read_line(&mut label_input)?;
                    let new_label = label_input.trim().to_string();
                    if !new_label.is_empty() {
                        let old_label = subwallet.label.clone().unwrap_or("(none)".to_string());
                        subwallet.label = Some(new_label.clone());
                        println!("✓ Label updated: '{}' → '{}'", old_label, new_label);
                        changes_made = true;
                    }
                }
                "2" => {
                    print!("Enter new notes (current: {}): ", subwallet.notes.as_deref().unwrap_or("(none)"));
                    io::stdout().flush()?;
                    let mut notes_input = String::new();
                    io::stdin().read_line(&mut notes_input)?;
                    let new_notes = notes_input.trim().to_string();
                    if !new_notes.is_empty() {
                        let old_notes = subwallet.notes.clone().unwrap_or("(none)".to_string());
                        subwallet.notes = Some(new_notes.clone());
                        println!("✓ Notes updated: '{}' → '{}'", old_notes, new_notes);
                        changes_made = true;
                    }
                }
                "3" => {
                    print!("Enter key=value pair (e.g., purpose=staking): ");
                    io::stdout().flush()?;
                    let mut data_input = String::new();
                    io::stdin().read_line(&mut data_input)?;
                    let input_str = data_input.trim();
                    if let Ok((key, value)) = parse_key_val(input_str) {
                        let old_value = subwallet.additional_data.insert(key.clone(), value.clone());
                        if let Some(old) = old_value {
                            println!("✓ Updated additional data '{}': '{}' → '{}'", key, old, value);
                        } else {
                            println!("✓ Added additional data '{}': '{}'", key, value);
                        }
                        changes_made = true;
                    } else {
                        println!("❌ Invalid format. Use key=value format.");
                    }
                }
                "4" => {
                    if subwallet.additional_data.is_empty() {
                        println!("No additional data to remove.");
                        continue;
                    }
                    println!("Current additional data:");
                    for (i, key) in subwallet.additional_data.keys().enumerate() {
                        println!("  {}. {}", i + 1, key);
                    }
                    print!("Enter key to remove: ");
                    io::stdout().flush()?;
                    let mut key_input = String::new();
                    io::stdin().read_line(&mut key_input)?;
                    let key = key_input.trim();
                    if subwallet.additional_data.remove(key).is_some() {
                        println!("✓ Removed additional data: '{}'", key);
                        changes_made = true;
                    } else {
                        println!("⚠️  Key '{}' not found", key);
                    }
                }
                "5" => {
                    print!("Enter type=address pair (e.g., legacy=1ABC...): ");
                    io::stdout().flush()?;
                    let mut addr_input = String::new();
                    io::stdin().read_line(&mut addr_input)?;
                    let input_str = addr_input.trim();
                    if let Ok((addr_type, address)) = parse_key_val(input_str) {
                        let old_address = subwallet.secondary_addresses.insert(addr_type.clone(), address.clone());
                        if let Some(old) = old_address {
                            println!("✓ Updated secondary address '{}': '{}' → '{}'", addr_type, old, address);
                        } else {
                            println!("✓ Added secondary address '{}': '{}'", addr_type, address);
                        }
                        changes_made = true;
                    } else {
                        println!("❌ Invalid format. Use type=address format.");
                    }
                }
                "6" => {
                    if subwallet.secondary_addresses.is_empty() {
                        println!("No secondary addresses to remove.");
                        continue;
                    }
                    println!("Current secondary addresses:");
                    for (i, addr_type) in subwallet.secondary_addresses.keys().enumerate() {
                        println!("  {}. {}", i + 1, addr_type);
                    }
                    print!("Enter address type to remove: ");
                    io::stdout().flush()?;
                    let mut type_input = String::new();
                    io::stdin().read_line(&mut type_input)?;
                    let addr_type = type_input.trim();
                    if subwallet.secondary_addresses.remove(addr_type).is_some() {
                        println!("✓ Removed secondary address type: '{}'", addr_type);
                        changes_made = true;
                    } else {
                        println!("⚠️  Address type '{}' not found", addr_type);
                    }
                }
                "7" => {
                    if subwallet.additional_data.is_empty() {
                        println!("No additional data to clear.");
                        continue;
                    }
                    print!("Clear all {} additional data entries? (y/N): ", subwallet.additional_data.len());
                    io::stdout().flush()?;
                    let mut confirm_input = String::new();
                    io::stdin().read_line(&mut confirm_input)?;
                    if confirm_input.trim().to_lowercase() == "y" {
                        let count = subwallet.additional_data.len();
                        subwallet.additional_data.clear();
                        println!("✓ Cleared {} additional data entries", count);
                        changes_made = true;
                    }
                }
                "8" => {
                    if subwallet.secondary_addresses.is_empty() {
                        println!("No secondary addresses to clear.");
                        continue;
                    }
                    print!("Clear all {} secondary addresses? (y/N): ", subwallet.secondary_addresses.len());
                    io::stdout().flush()?;
                    let mut confirm_input = String::new();
                    io::stdin().read_line(&mut confirm_input)?;
                    if confirm_input.trim().to_lowercase() == "y" {
                        let count = subwallet.secondary_addresses.len();
                        subwallet.secondary_addresses.clear();
                        println!("✓ Cleared {} secondary addresses", count);
                        changes_made = true;
                    }
                }
                "9" => break,
                _ => println!("❌ Invalid option. Please choose 1-9."),
            }
        }

        if !changes_made {
            println!("ℹ️  No modifications made.");
            return Ok(());
        }
    }

    // Save changes to database (subwallets use same method as wallets)
    let success = db.update_wallet(&subwallet)
        .context("Failed to update subwallet")?;

    if success {
        println!("\n🎉 Subwallet modified successfully!");
        println!("   Account: {}", args.account);
        println!("   Wallet Group: {}", args.wallet_group);
        println!("   Wallet: {}", args.wallet);
        println!("   Address Group: {}", args.address_group);
        if let Some(label) = &subwallet.label {
            println!("   Subwallet: {}", label);
        }
        println!("   Address: {}", subwallet.address);

        println!("\n💡 Next steps:");
        println!("   • View updated subwallet: wallet-backup show-subwallet --account \"{}\" --wallet-group \"{}\" --wallet \"{}\" --address-group \"{}\" --subwallet \"{}\"",
                 args.account, args.wallet_group, args.wallet, args.address_group, subwallet.label.as_deref().unwrap_or(&subwallet.address));
        println!("   • View address group: wallet-backup list-subwallets --account \"{}\" --wallet-group \"{}\" --wallet \"{}\" --address-group \"{}\"", args.account, args.wallet_group, args.wallet, args.address_group);
    } else {
        println!("\n❌ Failed to modify subwallet.");
    }

    Ok(())
}