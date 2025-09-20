use anyhow::Result;
use clap::Args;
use crate::blockchain::SupportedBlockchain;

#[derive(Args)]
pub struct ListCryptocurrenciesArgs {
    // No arguments needed - lists all supported cryptocurrencies
}

pub fn execute(_args: ListCryptocurrenciesArgs, _db: &crate::database::Database) -> Result<()> {
    println!("📋 Supported Cryptocurrencies");
    println!();

    // Get all supported blockchains
    let blockchains = [
        SupportedBlockchain::Bitcoin,
        SupportedBlockchain::Ethereum,
        SupportedBlockchain::Litecoin,
        SupportedBlockchain::Solana,
        SupportedBlockchain::Polygon,
        SupportedBlockchain::BinanceBNB,
        SupportedBlockchain::Optimism,
        SupportedBlockchain::Cronos,
        SupportedBlockchain::Cosmos,
        SupportedBlockchain::Polkadot,
        SupportedBlockchain::Sui,
        SupportedBlockchain::IOTA,
        SupportedBlockchain::Hedera,
        SupportedBlockchain::TON,
        SupportedBlockchain::XDC,
        SupportedBlockchain::Stellar,
        SupportedBlockchain::XRP,
        SupportedBlockchain::Cardano,
        SupportedBlockchain::Tron,
        SupportedBlockchain::Algorand,
    ];

    println!("💰 Available Blockchains ({} total):", blockchains.len());
    println!("   {:<15} {:<15} {:<20} {:<30}",
             "Name", "Coin Type", "Key Type", "Example Address");
    println!("   {}", "─".repeat(80));

    for blockchain in &blockchains {
        let coin_type = blockchain.get_coin_type()
            .map(|ct| ct.to_string())
            .unwrap_or_else(|| "N/A".to_string());

        let key_type = if blockchain.uses_ed25519() { "Ed25519" } else { "Secp256k1" };

        // Get example address format for each blockchain
        let example_address = get_example_address(blockchain);

        println!("   {:<15} {:<15} {:<20} {:<30}",
                 blockchain.to_string(),
                 coin_type,
                 key_type,
                 example_address);
    }

    println!("\n📈 Summary:");
    println!("   Total Supported: {} blockchains", blockchains.len());

    let secp256k1_count = blockchains.iter().filter(|b| !b.uses_ed25519()).count();
    let ed25519_count = blockchains.iter().filter(|b| b.uses_ed25519()).count();

    println!("   Secp256k1 Based: {} blockchains", secp256k1_count);
    println!("   Ed25519 Based: {} blockchains", ed25519_count);

    println!("\n💡 Usage:");
    println!("   • Add wallet: wallet-backup add-wallet --account \"<account>\" --wallet-group \"<group>\" --blockchain \"<blockchain>\" --name \"<name>\"");
    println!("   • Add standalone: wallet-backup add-standalone-wallet --private-key \"<key>\" --blockchain \"<blockchain>\" --name \"<name>\"");
    println!("   • Blockchain names are case-insensitive (e.g., 'bitcoin', 'Bitcoin', 'BITCOIN' all work)");

    Ok(())
}

fn get_example_address(blockchain: &SupportedBlockchain) -> &'static str {
    match blockchain {
        SupportedBlockchain::Bitcoin => "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
        SupportedBlockchain::Ethereum => "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
        SupportedBlockchain::Litecoin => "LTC1qw508d6qejxtdg4y5r3zarvary0c5xw7k",
        SupportedBlockchain::Solana => "11111111111111111111111111111112",
        SupportedBlockchain::Polygon => "0x8ba1f109551bD432803012645Hac136c",
        SupportedBlockchain::BinanceBNB => "bnb1grpf0955h0ykzq3ar5nmum7y6gdfl6lxfn46h2",
        SupportedBlockchain::Optimism => "0x4200000000000000000000000000000000000007",
        SupportedBlockchain::Cronos => "0x5C7F8A570d578ED84E63fdFA7b1eE72dEae1AE23",
        SupportedBlockchain::Cosmos => "cosmos1depk54cuajgkzea6zpgkq36tnjwdzv4ak663u6",
        SupportedBlockchain::Polkadot => "1Nh8FVZ1Ye4WGmGm3qJ3K5Zb5i5l2YjGbr2f3Td4c",
        SupportedBlockchain::Sui => "0x2::sui::SUI",
        SupportedBlockchain::IOTA => "iota1qpg9xjsj7sjhh7z5z8x2q3c7v9y8w7r6e5t4s3d2",
        SupportedBlockchain::Hedera => "0.0.123456",
        SupportedBlockchain::TON => "EQC-3ilVr-W0Uc3pLrh-2kHkTbHGiEMnHlPqr6VzJnHp8q0r",
        SupportedBlockchain::XDC => "xdc2f4f3b3c3a3e3d3c3b3a39383736353433323130",
        SupportedBlockchain::Stellar => "GCLWGQPMKXQSPF776IU33AH4PZNOOWNAWGGKVTBQMIC5IMKUNP3E6NVU",
        SupportedBlockchain::XRP => "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH",
        SupportedBlockchain::Cardano => "addr1vx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer5pnz75xxcrzqf96k",
        SupportedBlockchain::Tron => "TLPpkJWtjqHnRNz9ZFDojJ9TkNYSvZxyCB",
        SupportedBlockchain::Algorand => "KSDJALJDSAALJKDLAKJDLAKJDLAKJDLKASJDLKASJDLKASJDLKASJD",
    }
}