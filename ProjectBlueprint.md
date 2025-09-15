# Multi-Chain Wallet Backup Tool - Project Blueprint

## Project Overview

**Goal**: Create a self-contained, cross-platform command-line tool for multi-chain wallet backup and key management that provides users with complete local control over their cryptocurrency private keys and addresses.

**Core Purpose**: Self-sovereign wallet backup and key management tool (multi-chain, offline, local storage)

## Technical Specifications

### Language & Platform
- **Primary Language**: Rust
- **Target Platforms**: 
  - Linux (primary)
  - Windows 
  - macOS
  - Android (via NDK/Termux - stretch goal)
- **Database**: SQLite (local storage only)
- **Architecture**: Single binary, self-contained CLI application

### Implementation Status & Priority Order

**Implementation Strategy**: Working on 3 blockchains at a time for focused development and testing.

**Development Approach**: Prioritizing established Rust libraries over manual implementation to ensure reliability, maintainability, and compatibility with ecosystem standards. Libraries used:
- **Bitcoin**: `bitcoin = "0.32"` (official Bitcoin library)
- **Ethereum**: `alloy-primitives = "1.3.1"` (modern Alloy ecosystem, replaces ethers-rs)
- **Solana**: `solana-sdk = "3.0.0"` (official Solana SDK with proper Pubkey and Keypair types)
- **Stellar**: `stellar-base = "0.6.0"` (official Stellar library)
- **XRP**: `xrpl-rust = "1.0.0"` (official XRP library)
- **Litecoin**: Manual implementation (litcoin library uses Bitcoin network constants)
- **Cardano**: `pallas-crypto = "0.30"` and `pallas-codec = "0.30"` (Cardano ecosystem libraries)
- **TRON**: `anychain-tron = "0.2.13"` (official TRON library)
- **Polygon**: `alloy-primitives = "1.3.1"` (Ethereum-compatible)
- **Optimism**: `alloy-primitives = "1.3.1"` (Ethereum-compatible)
- **Cronos**: `alloy-primitives = "1.3.1"` (Ethereum-compatible)
- **Binance BNB**: `alloy-primitives = "1.3.1"` (Ethereum-compatible)
- **Cosmos**: `cosmrs = "0.22.0"` (official Cosmos SDK)
- **Algorand**: `algo_rust_sdk = "1.0.3"` (official Algorand SDK)
- **Hedera**: `hiero-sdk = "0.40.0"` (official Hedera SDK)
- **Polkadot**: Custom SS58 implementation using `blake2 = "0.10"` + `bs58 = "0.5"` (anychain-polkadot incompatible with ed25519-dalek v2.2.0)
- **Sui**: `sui-crypto = "0.0.7"` + `sui-sdk-types = "0.0.7"` (official Sui ecosystem)
- **IOTA**: `iota-crypto = "0.23.2"` + manual Bech32 encoding using `bech32 = "0.11"`
- **TON**: `tonlib-core = "0.26.1"` (mature TON core library with focused wallet functionality)

#### ✅ **Phase 0: Foundation (COMPLETED)**
- [x] **Bitcoin** - secp256k1, BIP-44 (m/44'/0'/0'/0/0) - *COMPLETE*
- [x] **Ethereum** - secp256k1, BIP-44 (m/44'/60'/0'/0/0) - *COMPLETE*
- [x] **Solana** - ed25519, SLIP-0010 (m/44'/501'/0'/0') - *COMPLETE*
- [x] **Stellar (XLM)** - ed25519, SLIP-0010 (m/44'/148'/0') - *COMPLETE*

#### ✅ **Phase 1: Implementation Complete (COMPLETED)**
- [x] **XRP (Ripple)** - secp256k1, BIP-44 (m/44'/144'/0'/0/0) - *COMPLETE*
- [x] **Litecoin** - secp256k1, BIP-44 (m/44'/2'/0'/0/0) - *COMPLETE*
- [x] **Cardano (ADA)** - ed25519, CIP-1852 (m/1852'/1815'/0'/0/0) - *COMPLETE*

#### ✅ **Phase 2: Complete (3/3 COMPLETE)**
- [x] **TRON** - secp256k1, T-prefixed Base58Check addresses (m/44'/195'/0'/0/0) - *COMPLETE*
- [x] **Polygon** - secp256k1, ETH-compatible (m/44'/966'/0'/0/0) - *COMPLETE*
- [x] **Optimism** - secp256k1, Uses ETH derivation (m/44'/60'/0'/0/0) - *COMPLETE*

#### ✅ **Phase 3: Complete (3/3 COMPLETE)**
- [x] **Cronos (CRO)** - secp256k1, BIP-44 (m/44'/394'/0'/0/0) - *COMPLETE*
- [x] **Binance BNB** - secp256k1, BEP-44 (m/44'/714'/0'/0/0) - *COMPLETE*
- [x] **Cosmos** - secp256k1, BIP-44 (m/44'/118'/0'/0/0) - *COMPLETE*

#### ✅ **Phase 4: Complete (3/3 COMPLETE)**
- [x] **Algorand** - ed25519, SLIP-0010 (m/44'/283'/0'/0'/0') - *COMPLETE*
- [x] **Hedera (HBAR)** - ed25519, SLIP-0010 (m/44'/3030'/0'/0'/0') - *COMPLETE*
- [x] **Polkadot** - ed25519, SLIP-0010 (m/44'/354'/0'/0'/0') - *COMPLETE*

#### 🚧 **Phase 5: In Progress (1/3 COMPLETE)**
- [x] **Sui** - ed25519, SLIP-0010 (m/44'/784'/0'/0'/0') - *COMPLETE*
- [ ] **IOTA** - ed25519, SLIP-0010 (m/44'/4218'/0'/0'/0')
- [ ] **TON** - ed25519, Custom derivation (m/44'/607'/0'/0')

#### 📋 **Phase 6: Final 2 (PENDING)**
- [ ] **XDC** - secp256k1, BIP-44 (m/44'/550'/0'/0/0)
- [ ] **Quant (QNT)** - secp256k1, BIP-44 (m/44'/1110'/0'/0/0)

**Current Focus**: Phase 5 In Progress - 17/21 blockchains implemented (81% complete). Sui Complete! Next: IOTA and TON

### Supported Blockchains Reference

| Blockchain | Coin Type (SLIP-0044) | Curve | Default Derivation Path | Notes |
|------------|----------------------|-------|-------------------------|-------|
| Bitcoin | 0 | secp256k1 | m/44'/0'/0'/0/0 | Standard BIP-44 |
| Ethereum | 60 | secp256k1 | m/44'/60'/0'/0/0 | Standard BIP-44 |
| Solana | 501 | ed25519 | m/44'/501'/0'/0' | Uses SLIP-0010 |
| Stellar (XLM) | 148 | ed25519 | m/44'/148'/0' | SLIP-0010 style |
| XRP (Ripple) | 144 | secp256k1 | m/44'/144'/0'/0/0 | BIP-44 compatible |
| Litecoin | 2 | secp256k1 | m/44'/2'/0'/0/0 | Standard BIP-44 |
| Cardano (ADA) | 1815 | ed25519 | m/1852'/1815'/0'/0/0 | Uses CIP-1852 |
| TRON | 195 | secp256k1 | m/44'/195'/0'/0/0 | T-prefixed Base58Check |
| Polygon | 966 | secp256k1 | m/44'/966'/0'/0/0 | ETH-compatible |
| Optimism | N/A | secp256k1 | m/44'/60'/0'/0/0 | Uses ETH derivation |
| Cronos (CRO) | 394 | secp256k1 | m/44'/394'/0'/0/0 | BIP-44 compatible |
| Binance BNB | 714 | secp256k1 | m/44'/714'/0'/0/0 | BEP-44 compatible |
| Cosmos | 118 | secp256k1 | m/44'/118'/0'/0/0 | Standard BIP-44 |
| Algorand | 283 | ed25519 | m/44'/283'/0'/0'/0' | SLIP-0010 style |
| Hedera (HBAR) | 3030 | ed25519 | m/44'/3030'/0'/0'/0' | SLIP-0010 style |
| Polkadot | 354 | ed25519 | m/44'/354'/0'/0'/0' | SLIP-0010 style |
| Sui | 784 | ed25519 | m/44'/784'/0'/0'/0' | SLIP-0010 style |
| IOTA | 4218 | ed25519 | m/44'/4218'/0'/0'/0' | SLIP-0010 style |
| TON | N/A | ed25519 | Custom | TON-specific derivation |
| XDC | 550 | secp256k1 | m/44'/550'/0'/0/0 | Standard BIP-44 |
| Quant (QNT) | 1110 | secp256k1 | m/44'/1110'/0'/0/0 | Standard |

### Key Standards Support
- **BIP-32**: Hierarchical Deterministic Wallets
- **BIP-39**: Mnemonic code for generating deterministic keys
- **BIP-44**: Multi-Account Hierarchy for Deterministic Wallets
- **SLIP-0010**: Universal private key derivation from master private key
- **SLIP-0044**: Registered coin types for BIP-44

## CLI Design

### Command Structure
```
wallet-backup <command> [options]
```

### Core Commands

#### 1. `import` - Add New Wallet
```bash
# From mnemonic (auto-infer derivation path)
wallet-backup import --mnemonic "word1 word2 ..." --blockchain ethereum --label "Main ETH"

# From mnemonic with passphrase
wallet-backup import --mnemonic "word1 word2 ..." --passphrase "extra-salt" --blockchain bitcoin --label "Cold BTC"

# From private key
wallet-backup import --private-key 0xabc123... --blockchain solana --label "Imported SOL"

# Custom derivation path (optional override)
wallet-backup import --mnemonic "..." --blockchain ethereum --custom-path "m/44'/60'/1'/0/0"
```

#### 2. `derive` - Generate Keys/Addresses
```bash
# Derive multiple addresses from mnemonic
wallet-backup derive --mnemonic "..." --blockchain ethereum --count 5

# Derive specific account/index
wallet-backup derive --mnemonic "..." --blockchain bitcoin --account 0 --index 10

# With passphrase
wallet-backup derive --mnemonic "..." --passphrase "salt" --blockchain ethereum --count 3
```

#### 3. `list` - Show All Wallets
```bash
wallet-backup list
```
Output:
```
Label         Blockchain   Address                                   Path
------------  -----------  ----------------------------------------  ----------------------
Main ETH      ethereum     0xABC123...                               m/44'/60'/0'/0/0
Cold BTC      bitcoin      1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa         m/44'/0'/0'/0/0
SOL Hot       solana       9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYt...  m/44'/501'/0'/0'
```

#### 4. `show` - Display Wallet Details
```bash
# Show by address
wallet-backup show --address 0xABC123

# Show by label
wallet-backup show --label "Main ETH"

# Include sensitive data (private key, mnemonic)
wallet-backup show --label "Main ETH" --include-sensitive
```

#### 5. `export` - Backup Data
```bash
# Export all wallets
wallet-backup export --format json --output backup.json

# Export specific wallet
wallet-backup export --label "Main ETH" --format json --include-sensitive

# Export to CSV
wallet-backup export --format csv --output wallets.csv
```

#### 6. `delete` - Remove Wallet
```bash
# Delete by address
wallet-backup delete --address 0xABC123

# Delete by label with confirmation skip
wallet-backup delete --label "Old Wallet" --force
```

#### 7. `tag` - Update Wallet Label
```bash
wallet-backup tag --address 0xABC123 --label "New Label"
```

#### 8. `search` - Find Wallets
```bash
# Search by term
wallet-backup search --term "btc"

# Search by blockchain
wallet-backup search --blockchain ethereum
```

## User Workflow

### Initial Setup
1. User installs single binary executable
2. First run creates local SQLite database
3. User imports existing wallets or generates new ones

### Typical Usage Patterns

#### Pattern 1: Import from Existing Mnemonic
```bash
# User has 12-word mnemonic from Trust Wallet
wallet-backup import --mnemonic "abandon abandon ... art" --blockchain ethereum --label "Trust ETH"
wallet-backup import --mnemonic "abandon abandon ... art" --blockchain bitcoin --label "Trust BTC"
wallet-backup import --mnemonic "abandon abandon ... art" --blockchain solana --label "Trust SOL"
```

#### Pattern 2: Generate Multiple Addresses
```bash
# Derive 10 addresses for receiving payments
wallet-backup derive --mnemonic "..." --blockchain ethereum --count 10
```

#### Pattern 3: Backup and Export
```bash
# Export all wallet data for backup
wallet-backup export --format json --include-sensitive --output full-backup.json

# Export public data only
wallet-backup list > public-addresses.txt
```

## Database Schema

### SQLite Table Structure

```sql
CREATE TABLE wallets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    label TEXT,                          -- User-friendly name
    blockchain TEXT NOT NULL,            -- e.g., 'ethereum', 'bitcoin'
    address TEXT NOT NULL UNIQUE,        -- Derived wallet address
    public_key TEXT,                     -- Derived public key
    private_key TEXT,                    -- Derived or imported private key
    mnemonic TEXT,                       -- BIP-39 seed phrase (if available)
    passphrase TEXT,                     -- Optional BIP-39 passphrase
    derivation_path TEXT NOT NULL,       -- Used derivation path
    account INTEGER,                     -- Account index (if from mnemonic)
    address_index INTEGER,               -- Address index (if from mnemonic)
    source_type TEXT NOT NULL,           -- 'mnemonic' or 'private_key'
    explorer_url TEXT,                   -- Blockchain explorer link
    imported_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    notes TEXT                           -- Optional user notes
);

-- Indexes for performance
CREATE INDEX idx_wallets_blockchain ON wallets(blockchain);
CREATE INDEX idx_wallets_label ON wallets(label);
CREATE INDEX idx_wallets_address ON wallets(address);
CREATE UNIQUE INDEX idx_wallets_address_unique ON wallets(address);
```

### Data Storage Rules

#### Required Fields
- `blockchain`: Must be supported blockchain identifier
- `address`: Unique wallet address
- `private_key`: Always required for wallet control
- `derivation_path`: Record which path was used
- `source_type`: Track how wallet was created

#### Optional Fields
- `mnemonic`: Only stored if wallet imported from seed phrase
- `passphrase`: Only if provided with mnemonic
- `label`: User-friendly identifier
- `account`/`address_index`: Only for mnemonic-derived wallets

#### Security Considerations
- Consider encrypting `private_key` and `mnemonic` fields
- Database stored locally only
- No network communication
- Optional: Add master password for database encryption

## Key Derivation Logic

### Derivation Path Inference
```rust
fn get_default_derivation_path(blockchain: &str, account: u32, address_index: u32) -> String {
    match blockchain {
        "bitcoin" => format!("m/44'/0'/{}'/{}/{}", account, 0, address_index),
        "ethereum" => format!("m/44'/60'/{}'/{}/{}", account, 0, address_index),
        "solana" => format!("m/44'/501'/{}/{}'", account, address_index),
        "cardano" => format!("m/1852'/1815'/{}'/{}/{}", account, 0, address_index),
        "xrp" => format!("m/44'/144'/{}'/{}/{}", account, 0, address_index),
        // ... additional chains
        _ => panic!("Unsupported blockchain: {}", blockchain)
    }
}
```

### Input Validation
- **Mnemonic**: Must be valid BIP-39 words (12, 15, 18, 21, or 24 words)
- **Private Key**: Must be valid hex string for the blockchain
- **Blockchain**: Must be in supported list
- **Derivation Path**: Must follow BIP-44/SLIP-0010 format if provided

## Security Features

### Local-Only Operation
- No network requests
- No RPC connections
- No balance queries
- All operations offline

### Data Protection
- Private keys never leave local system
- Optional database encryption
- Secure memory handling for sensitive data
- Optional: Secure deletion of sensitive memory

### Explorer Integration
- Generate read-only explorer URLs
- Support major block explorers per chain
- No API keys required

## Build & Distribution

### Build Targets
```bash
# Native compilation
cargo build --release

# Cross-compilation targets
cargo build --target x86_64-pc-windows-gnu     # Windows
cargo build --target x86_64-apple-darwin       # macOS
cargo build --target aarch64-linux-android     # Android
```

### Dependencies (Rust Crates)
- `clap`: CLI argument parsing
- `rusqlite`: SQLite database interface
- `bip39`: BIP-39 mnemonic handling
- `bitcoin`: Bitcoin key derivation
- `secp256k1`: secp256k1 curve operations
- `ed25519-dalek`: ed25519 curve operations
- `hex`: Hex encoding/decoding
- `serde`: Serialization for export
- `anyhow`: Error handling

### Binary Distribution
- Single static binary per platform
- No external dependencies
- Portable across systems
- Size target: <10MB per binary

## Future Enhancements

### Phase 2 Features
- Hardware wallet integration (Ledger/Trezor)
- Multi-signature wallet support
- Wallet import from various formats
- Advanced derivation path templates

### Phase 3 Features
- GUI version
- Mobile app versions
- Cloud backup (encrypted)
- Team/organization features

## Success Criteria

### Core Requirements Met
- ✅ Cross-platform single binary
- ✅ Support 10+ major blockchains
- ✅ BIP-39/44 and SLIP-0010 compliance
- ✅ Local SQLite storage
- ✅ Complete CLI interface
- ✅ Export/backup functionality

### User Experience Goals
- Simple enough for non-technical users
- Fast key derivation (<1 second per address)
- Clear error messages and validation
- Comprehensive help documentation
- Safe defaults (standard derivation paths)

This blueprint serves as the definitive reference for all implementation decisions and feature development.