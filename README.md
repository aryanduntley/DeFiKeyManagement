# DeFi Key Management Tool

A self-contained, cross-platform command-line tool for enterprise-grade hierarchical wallet management. Provides complete local control over cryptocurrency private keys across 20+ blockchains with advanced 5-level organizational structure.

## ✨ Features

### 🏗️ 5-Level Hierarchical Organization
```
Master Account → Wallet Group → Base Wallet → Address Group → Subwallets
     ↓              ↓              ↓              ↓           ↓
  TestAccount → MyPersonalWallet → MyBitcoinWallet → receiving → addr1, addr2, addr3
```

### Core Capabilities
- **20+ Native Blockchains**: Bitcoin, Ethereum, Solana, XRP, Stellar, Cardano, TRON, Polygon, Optimism, Cronos, Binance BNB, Cosmos, Algorand, Hedera, Polkadot, Sui, IOTA, TON, XDC, Litecoin
- **Hierarchical Deterministic (HD) Wallets**: Full BIP-32, BIP-39, BIP-44 compliance with proper key derivation
- **Enterprise Organization**: Multi-level hierarchy for complex portfolio management
- **Standalone Wallet Support**: Import individual private keys alongside HD wallets
- **Cross-Platform**: Single binary for Linux, Windows, macOS

### Security & Privacy
- **Local Storage**: All data stored locally in SQLite - no network requests or cloud dependencies
- **Self-Sovereign**: Complete control over private keys with no third-party access
- **Mnemonic Verification**: Cryptographic validation for all removal operations
- **Bottom-Up Security**: Only empty groups can be removed (prevents orphaned wallets)
- **Air-Gapped Capable**: Works completely offline

## 🚀 Installation

Download the appropriate binary for your platform from the releases page, or build from source:

```bash
git clone https://github.com/aryanduntley/DeFiKeyManagement.git
cd DeFiKeyManagement
cargo build --release
```

## 📘 CLI Commands Reference

### 🏛️ Master Account Management

#### Create Master Account
```bash
# From new mnemonic (auto-generated)
wallet-backup add-account --name "MyMainAccount"

# From existing mnemonic
wallet-backup add-account \
  --name "ImportedAccount" \
  --mnemonic "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"

# With passphrase
wallet-backup add-account \
  --name "SecureAccount" \
  --mnemonic "your mnemonic phrase here" \
  --passphrase "additional-security-phrase"
```

#### List Master Accounts
```bash
wallet-backup list-accounts
```

#### Show Master Account Details
```bash
# View account summary
wallet-backup show-account --account "MyMainAccount"

# View account with sensitive data (mnemonic phrase and passphrase)
wallet-backup show-account --account "MyMainAccount" --include-sensitive
```

#### Remove Master Account
```bash
# Removes entire hierarchy (requires mnemonic verification)
wallet-backup remove-account \
  --account "AccountToRemove" \
  --mnemonic "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
```

### 📁 Wallet Group Management

#### Create Wallet Group
```bash
wallet-backup add-wallet-group \
  --account "MyMainAccount" \
  --name "PersonalWallets" \
  --description "Personal cryptocurrency wallets"
```

#### List Wallet Groups
```bash
wallet-backup list-wallet-groups --account "MyMainAccount"
```

#### Show Wallet Group Details
```bash
# View wallet group with all wallets (public info only)
wallet-backup show-wallet-group \
  --account "MyMainAccount" \
  --group-name "PersonalWallets"

# View wallet group with private keys visible
wallet-backup show-wallet-group \
  --account "MyMainAccount" \
  --group-name "PersonalWallets" \
  --include-sensitive
```

#### Remove Wallet Group
```bash
# Only works if group is empty
wallet-backup remove-wallet-group \
  --account "MyMainAccount" \
  --wallet-group "EmptyGroup" \
  --mnemonic "your mnemonic phrase"
```

### 💰 Base Wallet Management

#### Add Base Wallet
```bash
# Create Bitcoin wallet
wallet-backup add-wallet \
  --account "MyMainAccount" \
  --wallet-group "PersonalWallets" \
  --blockchain "bitcoin" \
  --name "MyBitcoinWallet"

# Create Ethereum wallet with custom derivation
wallet-backup add-wallet \
  --account "MyMainAccount" \
  --wallet-group "TradingWallets" \
  --blockchain "ethereum" \
  --name "ETH-Trading" \
  --account-index 1
```

#### List Base Wallets
```bash
wallet-backup list-wallets \
  --account "MyMainAccount" \
  --wallet-group "PersonalWallets"
```

#### Show Base Wallet Details
```bash
# View wallet details (public info only)
wallet-backup show-wallet \
  --account "MyMainAccount" \
  --wallet-group "PersonalWallets" \
  --wallet "MyBitcoinWallet"

# View wallet with private key
wallet-backup show-wallet \
  --account "MyMainAccount" \
  --wallet-group "PersonalWallets" \
  --wallet "MyBitcoinWallet" \
  --include-sensitive
```

#### Modify Base Wallet Properties
```bash
# Interactive modification mode (default)
wallet-backup modify-wallet \
  --account "MyMainAccount" \
  --wallet-group "PersonalWallets" \
  --wallet-name "MyBitcoinWallet"

# Direct modification with flags
wallet-backup modify-wallet \
  --account "MyMainAccount" \
  --wallet-group "PersonalWallets" \
  --wallet-name "MyBitcoinWallet" \
  --label "Updated Bitcoin Wallet" \
  --notes "Primary trading wallet" \
  --add-data "exchange=binance" \
  --add-secondary "legacy=1BTC123..."

# Modify by address instead of name
wallet-backup modify-wallet \
  --account "MyMainAccount" \
  --wallet-group "PersonalWallets" \
  --address "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh" \
  --label "New Label"
```

#### Remove Base Wallet
```bash
wallet-backup remove-wallet \
  --account "MyMainAccount" \
  --wallet-group "PersonalWallets" \
  --wallet-name "OldWallet" \
  --mnemonic "your mnemonic phrase"
```

### 🗂️ Address Group Management

#### Create Address Group
```bash
wallet-backup add-address-group \
  --account "MyMainAccount" \
  --wallet-group "PersonalWallets" \
  --wallet "MyBitcoinWallet" \
  --name "receiving" \
  --description "Receiving addresses"
```

#### List Address Groups
```bash
wallet-backup list-address-groups \
  --account "MyMainAccount" \
  --wallet-group "PersonalWallets" \
  --wallet "MyBitcoinWallet"
```

#### Show Address Group Details
```bash
# View address group with all subwallets (public info only)
wallet-backup show-address-group \
  --account "MyMainAccount" \
  --wallet-group "PersonalWallets" \
  --wallet "MyBitcoinWallet" \
  --group-name "receiving"

# View address group with private keys visible
wallet-backup show-address-group \
  --account "MyMainAccount" \
  --wallet-group "PersonalWallets" \
  --wallet "MyBitcoinWallet" \
  --group-name "receiving" \
  --include-sensitive
```

#### Remove Address Group
```bash
# Only works if group is empty
wallet-backup remove-address-group \
  --account "MyMainAccount" \
  --wallet-group "PersonalWallets" \
  --wallet "MyBitcoinWallet" \
  --address-group "EmptyGroup" \
  --mnemonic "your mnemonic phrase"
```

### 🏠 Subwallet Management

#### Add Subwallet
```bash
wallet-backup add-subwallet \
  --account "MyMainAccount" \
  --wallet-group "PersonalWallets" \
  --wallet "MyBitcoinWallet" \
  --address-group "receiving" \
  --name "addr1"

# Multiple subwallets
wallet-backup add-subwallet \
  --account "MyMainAccount" \
  --wallet-group "PersonalWallets" \
  --wallet "MyBitcoinWallet" \
  --address-group "receiving" \
  --name "addr2"
```

#### List Subwallets
```bash
wallet-backup list-subwallets \
  --account "MyMainAccount" \
  --wallet-group "PersonalWallets" \
  --wallet "MyBitcoinWallet" \
  --address-group "receiving"
```

#### Show Subwallet Details
```bash
# View subwallet details (public info only)
wallet-backup show-subwallet \
  --account "MyMainAccount" \
  --wallet-group "PersonalWallets" \
  --wallet "MyBitcoinWallet" \
  --address-group "receiving" \
  --subwallet "addr1"

# View subwallet with private key
wallet-backup show-subwallet \
  --account "MyMainAccount" \
  --wallet-group "PersonalWallets" \
  --wallet "MyBitcoinWallet" \
  --address-group "receiving" \
  --subwallet "addr1" \
  --include-sensitive
```

#### Modify Subwallet Properties
```bash
# Interactive modification mode (default)
wallet-backup modify-subwallet \
  --account "MyMainAccount" \
  --wallet-group "PersonalWallets" \
  --wallet "MyBitcoinWallet" \
  --address-group "receiving" \
  --subwallet-name "addr1"

# Direct modification with flags
wallet-backup modify-subwallet \
  --account "MyMainAccount" \
  --wallet-group "PersonalWallets" \
  --wallet "MyBitcoinWallet" \
  --address-group "receiving" \
  --subwallet-name "addr1" \
  --label "Primary Receiving Address" \
  --notes "Used for customer payments" \
  --add-data "purpose=payments"

# Modify by address instead of name
wallet-backup modify-subwallet \
  --account "MyMainAccount" \
  --wallet-group "PersonalWallets" \
  --wallet "MyBitcoinWallet" \
  --address-group "receiving" \
  --address "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh" \
  --notes "Updated via address lookup"
```

#### Remove Subwallet
```bash
wallet-backup remove-subwallet \
  --account "MyMainAccount" \
  --wallet-group "PersonalWallets" \
  --wallet "MyBitcoinWallet" \
  --address-group "receiving" \
  --subwallet-name "addr1" \
  --mnemonic "your mnemonic phrase"
```

### 🔧 Standalone Wallet Management

#### Add Standalone Wallet
```bash
# Import from private key
wallet-backup add-standalone-wallet \
  --private-key "your-private-key-here" \
  --blockchain "ethereum" \
  --name "ImportedETH"
```

#### List Standalone Wallets
```bash
wallet-backup list-standalone-wallets
```

#### Show Standalone Wallet Details
```bash
# View standalone wallet by name (public info only)
wallet-backup show-standalone-wallet --name "ImportedETH"

# View standalone wallet by address (public info only)
wallet-backup show-standalone-wallet --address "0x742d35Cc6634C0532925a3b8D1b9f25C4e3DaF0"

# View standalone wallet with private key
wallet-backup show-standalone-wallet --name "ImportedETH" --include-sensitive
```

#### Modify Standalone Wallet Properties
```bash
# Interactive modification mode (default) - by name
wallet-backup modify-standalone-wallet --name "ImportedETH"

# Interactive modification mode (default) - by address
wallet-backup modify-standalone-wallet --address "0x742d35Cc6634C0532925a3b8D1b9f25C4e3DaF0"

# Direct modification with flags
wallet-backup modify-standalone-wallet \
  --name "ImportedETH" \
  --label "Coinbase Import" \
  --notes "Imported from Coinbase exchange" \
  --add-data "source=coinbase" \
  --add-data "import-date=2024-01-15" \
  --add-secondary "legacy=0x123..."
```

#### Remove Standalone Wallet
```bash
wallet-backup remove-standalone-wallet \
  --wallet-name "ImportedETH" \
  --private-key "your-private-key-here"
```

## 🏁 Quick Start: Complete 5-Level Hierarchy

### Create a Complete Portfolio
```bash
# 1. Create master account
wallet-backup add-account --account "Portfolio2024"

# 2. Create wallet group for personal funds
wallet-backup add-wallet-group \
  --account "Portfolio2024" \
  --name "PersonalFunds" \
  --description "Personal cryptocurrency portfolio"

# 3. Create Bitcoin base wallet
wallet-backup add-wallet \
  --account "Portfolio2024" \
  --wallet-group "PersonalFunds" \
  --blockchain "bitcoin" \
  --name "MainBitcoin"

# 4. Create address groups for organization
wallet-backup add-address-group \
  --account "Portfolio2024" \
  --wallet-group "PersonalFunds" \
  --wallet "MainBitcoin" \
  --name "receiving" \
  --description "Incoming payments"

wallet-backup add-address-group \
  --account "Portfolio2024" \
  --wallet-group "PersonalFunds" \
  --wallet "MainBitcoin" \
  --name "change" \
  --description "Change addresses"

# 5. Generate multiple subwallet addresses
wallet-backup add-subwallet \
  --account "Portfolio2024" \
  --wallet-group "PersonalFunds" \
  --wallet "MainBitcoin" \
  --address-group "receiving" \
  --name "payment1"

wallet-backup add-subwallet \
  --account "Portfolio2024" \
  --wallet-group "PersonalFunds" \
  --wallet "MainBitcoin" \
  --address-group "receiving" \
  --name "payment2"

# 6. View the complete hierarchy
wallet-backup show-wallet-group \
  --account "Portfolio2024" \
  --group-name "PersonalFunds"
```

### Create Multi-Blockchain Setup
```bash
# Add Ethereum wallet to same group
wallet-backup add-wallet \
  --account "Portfolio2024" \
  --wallet-group "PersonalFunds" \
  --blockchain "ethereum" \
  --name "MainEthereum"

# Create address group for DeFi
wallet-backup add-address-group \
  --account "Portfolio2024" \
  --wallet-group "PersonalFunds" \
  --wallet "MainEthereum" \
  --name "defi" \
  --description "DeFi protocol addresses"

# Generate DeFi addresses
wallet-backup add-subwallet \
  --account "Portfolio2024" \
  --wallet-group "PersonalFunds" \
  --wallet "MainEthereum" \
  --address-group "defi" \
  --name "uniswap"

wallet-backup add-subwallet \
  --account "Portfolio2024" \
  --wallet-group "PersonalFunds" \
  --wallet "MainEthereum" \
  --address-group "defi" \
  --name "compound"
```

## 🌐 Supported Blockchains

| Blockchain | Coin Type | Curve | Derivation Path | Status |
|------------|-----------|-------|-----------------|--------|
| Bitcoin | 0 | secp256k1 | m/44'/0'/0'/0/0 | ✅ |
| Ethereum | 60 | secp256k1 | m/44'/60'/0'/0/0 | ✅ |
| Solana | 501 | ed25519 | m/44'/501'/0'/0' | ✅ |
| Stellar (XLM) | 148 | ed25519 | m/44'/148'/0' | ✅ |
| XRP (Ripple) | 144 | secp256k1 | m/44'/144'/0'/0/0 | ✅ |
| Cardano (ADA) | 1815 | ed25519 | m/1852'/1815'/0'/0/0 | ✅ |
| TRON | 195 | secp256k1 | m/44'/195'/0'/0/0 | ✅ |
| Cronos (CRO) | 394 | secp256k1 | m/44'/394'/0'/0/0 | ✅ |
| TON | - | ed25519 | Custom | ✅ |
| Hedera (HBAR) | 3030 | ed25519 | m/44'/3030'/0'/0'/0' | ✅ |
| Algorand | 283 | ed25519 | m/44'/283'/0'/0'/0' | ✅ |
| Cosmos | 118 | secp256k1 | m/44'/118'/0'/0/0 | ✅ |
| Binance BNB | 714 | secp256k1 | m/44'/714'/0'/0/0 | ✅ |
| Litecoin | 2 | secp256k1 | m/44'/2'/0'/0/0 | ✅ |
| Polygon | 966 | secp256k1 | m/44'/966'/0'/0/0 | ✅ |
| Polkadot | 354 | ed25519 | m/44'/354'/0'/0'/0' | ✅ |
| Sui | 784 | ed25519 | m/44'/784'/0'/0'/0' | ✅ |
| Optimism | - | secp256k1 | m/44'/60'/0'/0/0 | ✅ |
| IOTA | 4218 | ed25519 | m/44'/4218'/0'/0'/0' | ✅ |
| XDC | 550 | secp256k1 | m/44'/550'/0'/0/0 | ✅ |

## 💾 Database Schema

The tool creates a local SQLite database (`wallets.db`) with a sophisticated 5-level hierarchical schema:

```sql
-- Master accounts (Level 1)
CREATE TABLE master_accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    mnemonic TEXT NOT NULL,
    master_private_key TEXT NOT NULL,
    passphrase TEXT DEFAULT '',
    next_account_index INTEGER DEFAULT 1,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Wallet groups (Level 2)
CREATE TABLE wallet_groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    master_account_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    account_index INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (master_account_id) REFERENCES master_accounts(id) ON DELETE CASCADE
);

-- Address groups (Level 3)
CREATE TABLE address_groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    wallet_group_id INTEGER NOT NULL,
    base_wallet_id INTEGER NOT NULL,
    blockchain TEXT NOT NULL,
    name TEXT NOT NULL,
    address_group_index INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (wallet_group_id) REFERENCES wallet_groups(id) ON DELETE CASCADE,
    FOREIGN KEY (base_wallet_id) REFERENCES wallets(id) ON DELETE CASCADE
);

-- Wallets (Levels 4 & 5: Base wallets and Subwallets)
CREATE TABLE wallets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    wallet_group_id INTEGER,
    address_group_id INTEGER,
    blockchain TEXT NOT NULL,
    address TEXT NOT NULL UNIQUE,
    public_key TEXT,
    private_key TEXT NOT NULL,
    derivation_path TEXT NOT NULL,
    address_index INTEGER,
    label TEXT,
    wallet_type TEXT NOT NULL DEFAULT 'hierarchical',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (wallet_group_id) REFERENCES wallet_groups(id) ON DELETE CASCADE,
    FOREIGN KEY (address_group_id) REFERENCES address_groups(id) ON DELETE CASCADE
);
```

## 🔐 Security Features

### Enterprise-Grade Security
- **Cryptographic Validation**: All removal operations require mnemonic verification
- **Bottom-Up Safety**: Empty-only group removal prevents orphaned wallets
- **Local-Only Operation**: No network requests, RPC connections, or balance queries
- **Air-Gapped Capable**: Works completely offline
- **Private Key Control**: Keys never leave your local system

### Graceful Error Handling
- **User-Friendly Messages**: Invalid mnemonics show helpful errors, not stack traces
- **Consistent Validation**: Same error handling patterns across all commands
- **Protective Safeguards**: Multiple confirmation layers for destructive operations

## 🔧 Interactive Modification Commands

### Hybrid Command Interface
All `modify-*` commands support both **interactive mode** (default) and **direct flag mode** for maximum usability:

#### Interactive Mode (Default)
When no modification flags are provided, commands enter an interactive menu:

```bash
wallet-backup modify-wallet --account "MyAccount" --wallet-group "Group1" --wallet "wallet1"

🔧 Interactive Modification Mode
Current wallet: MyBitcoinWallet

What would you like to modify?
1. Label (current: MyBitcoinWallet)
2. Notes (current: (none))
3. Add additional data
4. Remove additional data
5. Add secondary address
6. Remove secondary address
7. Clear all additional data (0 entries)
8. Clear all secondary addresses (0 addresses)
9. Exit

Choose option (1-9):
```

#### Direct Flag Mode
Provide modification flags for confirmed execution with verification step:

```bash
wallet-backup modify-wallet \
  --account "MyAccount" \
  --wallet-group "Group1" \
  --wallet "wallet1" \
  --label "New Label" \
  --add-data "exchange=binance"

📋 Planned Changes:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Label: 'MyBitcoinWallet' → 'New Label'
  Additional Data 'exchange': (none) → 'binance' (add)

Apply these changes? (Y/n): y
✓ Changes confirmed. Applying modifications...
```

### Safety & Verification Features
- **Direct Flag Verification**: All flag-based modifications show planned changes and require confirmation
- **Before/After Display**: Clear "old value → new value" format for all changes
- **Operation Indicators**: Shows whether operations are (add), (update), (remove), or (no change)
- **Detailed Impact**: Lists all affected entries when clearing data or secondary addresses
- **Cancellation Option**: Type 'n' or 'no' to cancel changes before application

### Interactive Features
- **Current Value Display**: Shows existing values for context
- **Key=Value Input**: Guided input for additional data and secondary addresses
- **Confirmation Prompts**: Safety confirmations for destructive operations in interactive mode
- **Numbered Lists**: Easy selection from existing data when removing items
- **Exit Anytime**: Choose option 9 to exit without saving

### Name/Address Flexibility
Many commands support `--name|address` pattern for flexible identification:

```bash
# Find wallet by name
wallet-backup show-standalone-wallet --name "MyWallet"

# Find same wallet by address
wallet-backup show-standalone-wallet --address "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh"

# Modify subwallet by name or address
wallet-backup modify-subwallet --account "A" --wallet-group "G" --wallet "W" --address-group "AG" --subwallet-name "addr1"
wallet-backup modify-subwallet --account "A" --wallet-group "G" --wallet "W" --address-group "AG" --address "bc1q..."
```

## 🔐 Sensitive Information Display

### --include-sensitive Flag
All show commands protect sensitive data by default. Use `--include-sensitive` to view private keys:

**Default Behavior (Safe)**:
- Master accounts: Shows mnemonic as "(use --include-sensitive to view)"
- Wallets/Subwallets: Shows private key as "(use --include-sensitive to view)"
- All other data (addresses, public keys, labels, notes) always visible

**With --include-sensitive**:
- Master accounts: Shows full mnemonic phrase and passphrase
- Wallets/Subwallets: Shows private key in plaintext

## 🏗️ Architecture Highlights

### Clean Command Structure
- **Hierarchical Navigation**: All commands follow the same 5-level pattern
- **Consistent Parameters**: Same `--account`, `--wallet-group`, `--wallet` pattern
- **Flexible Identification**: Support for both name and address-based lookups
- **Interactive by Default**: User-friendly guided modification with fallback to direct flags
- **Defensive Operations**: Safety checks prevent accidental data loss

### Database Integrity
- **Foreign Key Constraints**: Automatic cascading with SQLite `ON DELETE CASCADE`
- **Referential Integrity**: All relationships properly maintained
- **Clean Separation**: CLI handles validation, database handles operations

## 📊 Development Status

**Current Version**: 2.0.0 (Complete 5-Level Hierarchical System)

### ✅ Completed
- **5-Level Hierarchy**: Master Account → Wallet Group → Base Wallet → Address Group → Subwallet
- **30+ CLI Commands**: Complete command set including show and modify commands for all levels
- **Interactive Modification Interface**: Hybrid interactive/direct flag approach for all modify commands
- **Flexible Identification**: --name|address support where appropriate for easy wallet lookup
- **Sensitive Data Protection**: --include-sensitive flag system for secure information display
- **20+ Blockchain Support**: All major cryptocurrencies with native key derivation
- **Security Architecture**: Mnemonic validation, empty-group-only removal, graceful errors
- **Database Schema**: Full foreign key relationships with cascading support
- **Comprehensive Testing**: All hierarchy levels and safety features tested

### 🔄 Architecture Improvements
- **Graceful BIP-39 Validation**: User-friendly error messages for invalid mnemonics
- **Consistent Error Handling**: Same validation patterns across all commands
- **Clean Database Layer**: Separation of concerns between CLI and database operations

## 🛠 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

## ⚠️ Disclaimer

This tool is for educational and personal backup purposes. Always verify generated addresses and private keys against trusted sources before using with real funds. Keep your mnemonic phrases and private keys secure and never share them with anyone.

## 📚 Useful Links

- [BIP-32 Specification](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki)
- [BIP-39 Word Lists](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki)
- [BIP-44 Multi-Account Hierarchy](https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki)
- [SLIP-0044 Coin Types](https://github.com/satoshilabs/slips/blob/master/slip-0044.md)
- [Project Repository](https://github.com/aryanduntley/DeFiKeyManagement)