# DeFi Key Management Tool

A self-contained, cross-platform command-line tool for multi-chain wallet backup and key management. Provides users with complete local control over their cryptocurrency private keys and addresses across 20+ blockchains.

## =€ Features

- **Multi-Chain Support**: Bitcoin, Ethereum, Solana, XRP, Stellar, Cardano, TRON, and 15+ more blockchains
- **BIP Standards Compliant**: Full support for BIP-32, BIP-39, BIP-44, SLIP-0010, and SLIP-0044
- **Cross-Platform**: Single binary for Linux, Windows, macOS (Android support planned)
- **Local Storage**: All data stored locally in SQLite - no network requests or cloud dependencies
- **Self-Sovereign**: Complete control over private keys with no third-party access
- **Import Flexibility**: Support for mnemonic phrases (with optional passphrases) and private keys
- **Export Capabilities**: JSON and CSV export formats for backup purposes

## =' Installation

Download the appropriate binary for your platform from the releases page, or build from source:

```bash
git clone https://github.com/aryanduntley/DeFiKeyManagement.git
cd DeFiKeyManagement
cargo build --release
```

## =Ö Quick Start

### Import from Mnemonic Phrase
```bash
# Import wallet from Trust Wallet or other multi-wallet mnemonic
./wallet-backup import --mnemonic "word1 word2 ... word12" --blockchain ethereum --label "Main ETH"
./wallet-backup import --mnemonic "word1 word2 ... word12" --blockchain bitcoin --label "Main BTC"
./wallet-backup import --mnemonic "word1 word2 ... word12" --blockchain solana --label "Main SOL"
```

### Import from Private Key
```bash
./wallet-backup import --private-key 0xabc123... --blockchain ethereum --label "Imported ETH"
```

### Generate Multiple Addresses
```bash
# Derive 10 addresses from your mnemonic for receiving payments
./wallet-backup derive --mnemonic "word1 word2 ..." --blockchain ethereum --count 10
```

### List All Wallets
```bash
./wallet-backup list
```

### View Wallet Details
```bash
./wallet-backup show --label "Main ETH"
./wallet-backup get "Main ETH"  # Shorthand version
```

### Export for Backup
```bash
# Export all wallets (public data only)
./wallet-backup export --format json --output backup.json

# Export with sensitive data (private keys/mnemonics)
./wallet-backup export --format json --include-sensitive --output full-backup.json
```

## < Supported Blockchains

| Blockchain | Coin Type | Curve | Derivation Path | Status |
|------------|-----------|-------|-----------------|--------|
| Bitcoin | 0 | secp256k1 | m/44'/0'/0'/0/0 |  |
| Ethereum | 60 | secp256k1 | m/44'/60'/0'/0/0 |  |
| Solana | 501 | ed25519 | m/44'/501'/0'/0' |  |
| Stellar (XLM) | 148 | ed25519 | m/44'/148'/0' |  * |
| XRP (Ripple) | 144 | secp256k1 | m/44'/144'/0'/0/0 | = |
| Cardano (ADA) | 1815 | ed25519 | m/1852'/1815'/0'/0/0 | = |
| TRON | 195 | secp256k1 | m/44'/195'/0'/0/0 | = |
| Cronos (CRO) | 394 | secp256k1 | m/44'/394'/0'/0/0 | = |
| TON | - | ed25519 | Custom | = |
| Hedera (HBAR) | 3030 | ed25519 | m/44'/3030'/0'/0'/0' | = |
| Algorand | 283 | ed25519 | m/44'/283'/0'/0'/0' | = |
| Cosmos | 118 | secp256k1 | m/44'/118'/0'/0/0 | = |
| Binance BNB | 714 | secp256k1 | m/44'/714'/0'/0/0 | = |
| Litecoin | 2 | secp256k1 | m/44'/2'/0'/0/0 | = |
| Polygon | 966 | secp256k1 | m/44'/966'/0'/0/0 | = |
| Polkadot | 354 | ed25519 | m/44'/354'/0'/0'/0' | = |
| Sui | 784 | ed25519 | m/44'/784'/0'/0'/0' | = |
| Optimism | - | secp256k1 | m/44'/60'/0'/0/0 | = |
| IOTA | 4218 | ed25519 | m/44'/4218'/0'/0'/0' | = |
| XDC | 550 | secp256k1 | m/44'/550'/0'/0/0 | = |
| Quant (QNT) | 1110 | secp256k1 | m/44'/1110'/0'/0/0 | = |

**Legend**:  Implemented |   Implemented but needs validation | = In development

*Stellar implementation requires integration with stellar-base library for address validation

## =à Commands Reference

### Core Commands
- `import` - Add new wallet from mnemonic or private key
- `derive` - Generate multiple keys/addresses from mnemonic
- `list` - Display all stored wallets in table format
- `show` - Show detailed wallet information
- `get <label>` - Quick wallet retrieval by label (shorthand)
- `export` - Export wallets to JSON/CSV format
- `delete` - Remove wallet from database
- `tag` - Update wallet label
- `search` - Find wallets by term/blockchain

### Command Options
- `--mnemonic` - 12-24 word BIP-39 mnemonic phrase
- `--passphrase` - Optional BIP-39 passphrase (25th word)
- `--private-key` - Import from existing private key
- `--blockchain` - Target blockchain (bitcoin, ethereum, solana, etc.)
- `--label` - User-friendly wallet name
- `--count N` - Number of addresses to derive
- `--include-sensitive` - Include private keys in export
- `--format` - Export format (json, csv)

## = Security Features

- **Local-Only Operation**: No network requests, RPC connections, or balance queries
- **Air-Gapped Capable**: Works completely offline
- **Private Key Control**: Keys never leave your local system
- **SQLite Storage**: Local database with optional encryption
- **Explorer Links**: Generate read-only blockchain explorer URLs (no API keys required)

## =Á Database Schema

The tool creates a local SQLite database (`wallets.db`) with the following structure:

```sql
CREATE TABLE wallets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    label TEXT,
    blockchain TEXT NOT NULL,
    address TEXT NOT NULL UNIQUE,
    public_key TEXT,
    private_key TEXT,
    mnemonic TEXT,
    passphrase TEXT,
    derivation_path TEXT NOT NULL,
    account INTEGER,
    address_index INTEGER,
    source_type TEXT NOT NULL,
    explorer_url TEXT,
    imported_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    notes TEXT
);
```

## <× Development Status

**Current Version**: 0.75.0 (75% Complete)

###  Completed
- CLI framework with all 8 commands
- SQLite database layer with full CRUD operations
- Bitcoin, Ethereum, Solana, Stellar blockchain handlers
- BIP-32, BIP-39, SLIP-0010 cryptographic implementations
- Multi-mnemonic and multi-account wallet support
- Cross-platform compilation support

### = In Progress
- Remaining 17+ blockchain handler implementations
- Stellar address validation with stellar-base integration
- Comprehensive test suite with known test vectors
- Cross-platform binary builds

### =Ë Planned
- Hardware wallet integration (Ledger/Trezor)
- GUI version
- Mobile app versions
- Advanced derivation path templates

## > Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## =Ä License

MIT License - see [LICENSE](LICENSE) file for details.

##   Disclaimer

This tool is for educational and personal backup purposes. Always verify generated addresses and private keys against trusted sources before using with real funds. Keep your mnemonic phrases and private keys secure and never share them with anyone.

## = Useful Links

- [BIP-32 Specification](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki)
- [BIP-39 Word Lists](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki)
- [SLIP-0044 Coin Types](https://github.com/satoshilabs/slips/blob/master/slip-0044.md)