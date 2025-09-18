# Implementation Plan: 4-Level Hierarchical Wallet Architecture

**Project**: DeFi Key Management Tool
**Version**: 2.0 - Hierarchical Redesign
**Date**: 2025-09-17
**Reference**: Progress4.json

## Overview

This implementation plan outlines the complete migration from the current 2-level structure (WalletGroup → WalletRecord) to a 4-level hierarchical architecture (MasterAccount → WalletGroup → AddressGroup → WalletAddress) with complete auto-increment control and improved security.

---

## Phase 1: Database Schema Migration

### 1.1 New Database Structures
- [ ] Create `MasterAccount` struct with auto-increment tracking
- [ ] Create `WalletGroup` struct with auto-assigned account indexes
- [ ] Create `AddressGroup` struct for blockchain-specific address collections
- [ ] Create `WalletAddress` struct with dual references (group_id + address_group_id)
- [ ] Preserve existing `WalletAdditionalData` and `WalletSecondaryAddresses` structs

### 1.2 Database Schema Implementation

#### 1.2.1 Core Tables
- [ ] **`master_accounts` table**
  ```sql
  CREATE TABLE master_accounts (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      name TEXT UNIQUE NOT NULL,
      mnemonic TEXT NOT NULL,
      master_private_key TEXT NOT NULL,
      passphrase TEXT,
      next_account_index INTEGER DEFAULT 0,
      created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
      updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
  );
  ```

- [ ] **`wallet_groups` table**
  ```sql
  CREATE TABLE wallet_groups (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      master_account_id INTEGER NOT NULL,
      name TEXT NOT NULL,
      description TEXT,
      account_index INTEGER NOT NULL,
      created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
      updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
      FOREIGN KEY (master_account_id) REFERENCES master_accounts(id) ON DELETE CASCADE,
      UNIQUE(master_account_id, name),
      UNIQUE(master_account_id, account_index)
  );
  ```

- [ ] **`address_groups` table**
  ```sql
  CREATE TABLE address_groups (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      wallet_group_id INTEGER NOT NULL,
      blockchain TEXT NOT NULL,
      name TEXT NOT NULL,
      address_group_index INTEGER NOT NULL,
      next_address_index INTEGER DEFAULT 0,
      created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
      updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
      FOREIGN KEY (wallet_group_id) REFERENCES wallet_groups(id) ON DELETE CASCADE,
      UNIQUE(wallet_group_id, name), -- Address group names must be unique within wallet group (not just per blockchain)
      UNIQUE(wallet_group_id, blockchain, address_group_index)
  );
  ```

- [ ] **`wallet_addresses` table**
  ```sql
  CREATE TABLE wallet_addresses (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      wallet_group_id INTEGER, -- NULL for private_key-only wallets (orphaned entries)
      address_group_id INTEGER, -- NULL for private_key-only wallets (orphaned entries)
      blockchain TEXT NOT NULL,
      address TEXT UNIQUE NOT NULL,
      address_with_checksum TEXT,
      private_key TEXT NOT NULL,
      public_key TEXT,
      derivation_path TEXT, -- NULL for private_key-only wallets (no derivation)
      address_index INTEGER, -- NULL for private_key-only wallets (no derivation sequence)
      label TEXT, -- Individual address label (empty by default)
      source_type TEXT NOT NULL DEFAULT 'mnemonic', -- "mnemonic" or "private_key"
      explorer_url TEXT,
      notes TEXT,
      created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
      FOREIGN KEY (wallet_group_id) REFERENCES wallet_groups(id) ON DELETE CASCADE,
      FOREIGN KEY (address_group_id) REFERENCES address_groups(id) ON DELETE CASCADE,
      UNIQUE(address_group_id, address_index) -- Only applies when both are NOT NULL
  );
  ```

#### 1.2.2 Preserved Metadata Tables
- [ ] Keep existing `wallet_additional_data` table (links to wallet_addresses.id)
  - Purpose: Blockchain-specific metadata (Hedera key_type, Bitcoin address_type, etc.)
- [ ] Keep existing `wallet_secondary_addresses` table (links to wallet_addresses.id)
  - Purpose: Alternative address formats (EVM, legacy, P2SH, etc.)
- [ ] **Note**: `additional_data` and `secondary_addresses` HashMap fields in WalletAddress struct are loaded from these tables

#### 1.2.3 Database Indexes
- [ ] Add performance indexes on all foreign keys
- [ ] Add indexes for common query patterns (account lookups, blockchain filtering)

### 1.3 Database Operations Implementation

#### 1.3.1 Master Account Operations
- [ ] `create_master_account(name, mnemonic, passphrase) -> Result<i64>`
- [ ] `get_master_account_by_name(name) -> Result<Option<MasterAccount>>`
- [ ] `list_master_accounts() -> Result<Vec<MasterAccount>>`
- [ ] `delete_master_account(name, mnemonic_verification) -> Result<bool>`

#### 1.3.2 Wallet Group Operations
- [ ] `create_wallet_group(master_account_id, name, description) -> Result<i64>`
- [ ] `get_wallet_group_by_name(master_account_id, name) -> Result<Option<WalletGroup>>`
- [ ] `list_wallet_groups(master_account_id) -> Result<Vec<WalletGroup>>`
- [ ] `rename_wallet_group(master_account_id, old_name, new_name) -> Result<bool>`
- [ ] `delete_wallet_group(master_account_id, group_name, mnemonic_verification) -> Result<bool>`

#### 1.3.3 Address Group Operations
- [ ] `create_address_group(wallet_group_id, blockchain, name) -> Result<i64>`
- [ ] `get_or_create_default_address_group(wallet_group_id, blockchain) -> Result<i64>` (creates "btc-0" style)
- [ ] `list_address_groups(wallet_group_id, blockchain) -> Result<Vec<AddressGroup>>`
- [ ] `rename_address_group(wallet_group_id, blockchain, old_name, new_name) -> Result<bool>`
- [ ] `delete_address_group(wallet_group_id, blockchain, name, mnemonic_verification) -> Result<bool>`

#### 1.3.4 Wallet Address Operations
- [ ] `create_wallet_address(address_group_id, wallet_data) -> Result<i64>` (for mnemonic-derived addresses)
- [ ] `create_orphaned_wallet_address(wallet_data) -> Result<i64>` (for private_key-only addresses, NULL group IDs)
- [ ] `get_wallet_addresses_by_group(wallet_group_id) -> Result<Vec<WalletAddress>>`
- [ ] `get_wallet_addresses_by_address_group(address_group_id) -> Result<Vec<WalletAddress>>`
- [ ] `get_orphaned_wallet_addresses() -> Result<Vec<WalletAddress>>` (WHERE wallet_group_id IS NULL)
- [ ] `delete_wallet_address(address_group_id, address_name, mnemonic_verification) -> Result<bool>`
- [ ] `delete_orphaned_wallet_address(address_id) -> Result<bool>` (no mnemonic verification needed)

#### 1.3.5 Bulk Operations (for import-multi)
- [ ] `create_complete_hierarchy_from_mnemonic(account_name, wallet_group_name, blockchain_list, mnemonic) -> Result<HierarchyResult>`
- [ ] Helper function to create master account + wallet group + address groups + initial addresses in single transaction
- [ ] Returns structure with all created IDs and addresses for confirmation display

---

## Phase 2: CLI Command Implementation

### 2.1 Command Structure Changes

#### 2.1.1 Remove/Rename Existing Commands
- [ ] **RENAME**: `derive-multi` → `add-blockchain`
- [ ] **REMOVE**: Manual account/address index parameters from all commands
- [ ] **ADD**: `--account-name` parameter requirement to all commands

#### 2.1.2 New Master Account Management Commands
- [ ] **`create-master`**
  ```bash
  create-master --account-name "MyMasterAccount1" --mnemonic "twelve word phrase..."
  ```
  - Args: `account_name: String, mnemonic: String, passphrase: Option<String>`
  - Auto-assigns `next_account_index = 0`

- [ ] **`list-masters`**
  ```bash
  list-masters
  ```
  - No args, shows all master accounts with basic info

### 2.2 Wallet Group Management Commands

#### 2.2.1 Wallet Group Commands
- [ ] **`add-wallet-group`**
  ```bash
  add-wallet-group --account "MyMasterAccount1" --name "PersonalWallet"
  ```
  - Args: `account: String, name: String, description: Option<String>`
  - Auto-assigns next account index

- [ ] **`rename-wallet-group`**
  ```bash
  rename-wallet-group --account "MyMasterAccount1" --old-name "MetaMask_Main" --new-name "MetaMask_Primary"
  ```
  - Args: `account: String, old_name: String, new_name: String`

- [ ] **`list-wallet-groups`**
  ```bash
  list-wallet-groups --account "MyMasterAccount1"
  ```
  - Args: `account: String`
  - Shows: Group name, account index, address group count, total addresses

- [ ] **`remove-wallet-group`**
  ```bash
  remove-wallet-group --account "MyMasterAccount1" --wallet-group "PersonalWallet" --mnemonic "verification phrase"
  ```
  - Args: `account: String, wallet_group: String, mnemonic: String`
  - Requires 'I'm sure' confirmation
  - Cascades to remove all address groups and addresses

### 2.3 Blockchain and Address Group Management

#### 2.3.1 Blockchain Commands
- [ ] **`add-blockchain`** (creates default address groups)
  ```bash
  add-blockchain --account "MyMasterAccount1" --wallet-group "PersonalWallet" --blockchains "cardano,polkadot"
  ```
  - Args: `account: String, wallet_group: String, blockchains: String`
  - **REMOVED**: `mnemonic` parameter (already stored)
  - Auto-creates default address groups with blockchain-specific names: "cardano-0", "polkadot-0"

#### 2.3.2 Address Group Commands
- [ ] **`add-address-group`**
  ```bash
  add-address-group --account "MyMasterAccount1" --wallet-group "PersonalWallet" --blockchain "bitcoin" --name "Trading"
  ```
  - Args: `account: String, wallet_group: String, blockchain: String, name: String`
  - **REQUIRED**: `name` parameter must be unique within wallet-group + blockchain combination
  - Allows multiple address groups per blockchain (e.g., "btc-savings", "btc-trading", "btc-cold")

- [ ] **`rename-address-group`**
  ```bash
  rename-address-group --account "MyMasterAccount1" --wallet-group "PersonalWallet" --old-name "btc-0" --new-name "HotWallet"
  ```
  - Args: `account: String, wallet_group: String, old_name: String, new_name: String`

- [ ] **`remove-address-group`**
  ```bash
  remove-address-group --account "MyMasterAccount1" --wallet-group "PersonalWallet" --address-group "Trading" --mnemonic "verification phrase"
  ```
  - Args: `account: String, wallet_group: String, address_group: String, mnemonic: String`
  - Requires 'I'm sure' confirmation
  - Removes all addresses within that address group
  - **NOTE**: This replaces `remove-blockchain` - user must specify exact address group to remove

### 2.4 Individual Address Management

#### 2.4.1 Address Commands
- [ ] **`add-address`**
  ```bash
  add-address --account "MyMasterAccount1" --wallet-group "PersonalWallet" --address-group "Trading" --name "TradingAddr1"
  ```
  - Args: `account: String, wallet_group: String, address_group: String, name: String`
  - Creates new individual address within existing address group
  - Auto-assigns next address index within the address group

- [ ] **`rename-address`** (rename individual address)
  ```bash
  rename-address --account "MyMasterAccount1" --wallet-group "PersonalWallet" --address-group "Trading" --old-name "TradingAddr1" --new-name "MainTradingWallet"
  ```
  - **Alternative syntax**: Use actual address instead of name
  ```bash
  rename-address --account "MyMasterAccount1" --wallet-group "PersonalWallet" --address-group "Trading" --address "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa" --new-name "MainTradingWallet"
  ```
  - Args: `account: String, wallet_group: String, address_group: String, old_name_or_address: String, new_name: String`

- [ ] **`remove-address`**
  ```bash
  remove-address --account "MyMasterAccount1" --wallet-group "PersonalWallet" --address-group "Trading" --address-name "TradingAddr1" --mnemonic "verification phrase"
  ```
  - **Alternative syntax**: Use actual address instead of name
  ```bash
  remove-address --account "MyMasterAccount1" --wallet-group "PersonalWallet" --address-group "Trading" --address "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa" --mnemonic "verification phrase"
  ```
  - Args: `account: String, wallet_group: String, address_group: String, address_name_or_address: String, mnemonic: String`
  - Requires 'I'm sure' confirmation

### 2.5 Listing Commands

#### 2.5.1 Refined Listing Commands
- [ ] **`list-cryptocurrencies`**
  ```bash
  list-cryptocurrencies
  ```
  - Shows all supported blockchains with coin types and derivation info

- [ ] **`list-wallet-groups`**
  ```bash
  list-wallet-groups --account "MyMasterAccount1"
  ```
  - Args: `account: String`
  - Shows: Group name, account index, total address groups, total addresses

- [ ] **`list-address-groups`**
  ```bash
  list-address-groups --account "MyMasterAccount1" --wallet-group "PersonalWallet" --blockchain "bitcoin"
  ```
  - **Optional blockchain**: If omitted, shows all blockchains
  ```bash
  list-address-groups --account "MyMasterAccount1" --wallet-group "PersonalWallet"
  ```
  - Args: `account: String, wallet_group: String, blockchain: Option<String>`
  - Shows: Address group names with address counts (e.g., "btc-0 (3 addresses)", "btc-trading (5 addresses)")

- [ ] **`list-addresses`**
  ```bash
  list-addresses --account "MyMasterAccount1" --wallet-group "PersonalWallet" --address-group "btc-trading"
  ```
  - Args: `account: String, wallet_group: String, address_group: String`
  - Shows: Individual addresses with names, address_index, derivation paths, and actual addresses

- [ ] **`list-orphaned-wallets`**
  ```bash
  list-orphaned-wallets
  ```
  - No args - shows all private_key-only wallets (source_type = "private_key", wallet_group_id IS NULL)
  - Useful for managing wallets imported without mnemonics

### 2.6 Legacy Command Analysis & Remapping

#### 2.6.1 Commands Requiring Complete Remapping
- [ ] **`import-multi`** ✅ **REMAP TO NEW 4-LEVEL HIERARCHY**
  - Add `--account` parameter requirement
  - Auto-creates master account if doesn't exist
  - Auto-creates wallet group with specified name
  - Auto-creates default address groups for each blockchain (e.g., "bitcoin-0", "ethereum-0", etc.)
  - Auto-creates first address (index 0) in each address group
  - Only works with mnemonic (source_type = "mnemonic")
  - **CRITICAL**: Must be completely remapped to new functionality - creates entire 4-level hierarchy from single command

- [ ] **`show-group`** ✅ **REMAP TO show-wallet-group**
  - **RENAME**: `show-group` → `show-wallet-group` for clarity
  - Add `--account` parameter requirement
  - Show account index, address groups, and address counts
  - Display hierarchical structure clearly
  - **NOTE**: Complete restructuring needed to display 4-level hierarchy

#### 2.6.2 Commands Being Replaced by New Fine-Grained Commands
- [ ] **`derive`** ❌ **DEPRECATED - REPLACE WITH NEW COMMANDS**
  - **REASON**: `derive` command is too generic and risky with --account and --index manual options
  - **REPLACED BY**: `add-address-group` and `add-address` commands provide fine control
  - **BENEFIT**: No risk of index conflicts, proper hierarchy management
  - **STATUS**: Mark as deprecated, suggest migration to new commands

- [ ] **`list-groups`** ❌ **DEPRECATED - REPLACED**
  - **REPLACED BY**: `list-wallet-groups --account "AccountName"`
  - **REASON**: New command requires account context and is more specific
  - **STATUS**: Remove or redirect to new command

- [ ] **`rename-group`** ❌ **DEPRECATED - REPLACED**
  - **REPLACED BY**: `rename-wallet-group --account "AccountName" --old-name "..." --new-name "..."`
  - **REASON**: New command requires account context for proper hierarchy navigation
  - **STATUS**: Remove or redirect to new command

#### 2.6.3 Commands Requiring Major Redesign (To Be Addressed)
- [ ] **`list`** ⚠️ **NEEDS COMPREHENSIVE REDESIGN**
  - **ISSUE**: Too generic for new 4-level hierarchy
  - **REPLACED BY**: Fine-grained list commands (list-wallet-groups, list-address-groups, list-addresses)
  - **STATUS**: Requires comprehensive solution - redesign needed
  - **TODO**: Define new behavior or deprecate in favor of specific list commands

- [ ] **`get`** ⚠️ **NEEDS REDEFINITION**
  - **ISSUE**: Too generic and doesn't suit new hierarchical system
  - **PROBLEM**: Current `get <name>` syntax doesn't specify which level (account/group/address-group/address)
  - **STATUS**: Requires comprehensive redefinition
  - **TODO**: Define new behavior with proper hierarchy navigation

- [ ] **`export`** ⚠️ **NEEDS COMPREHENSIVE SOLUTION**
  - **ISSUE**: Current export doesn't account for 4-level hierarchy
  - **NEEDS**: Export at different levels (account, wallet-group, address-group, individual address)
  - **STATUS**: Requires redesign for hierarchical data structures
  - **TODO**: Define export formats and hierarchy levels

- [ ] **`delete`** ⚠️ **NEEDS COMPREHENSIVE SOLUTION**
  - **ISSUE**: Current delete is ambiguous in 4-level hierarchy
  - **REPLACED BY**: Specific remove commands (remove-account, remove-wallet-group, remove-address-group, remove-address)
  - **STATUS**: Likely deprecated in favor of specific remove commands
  - **TODO**: Determine if keep as alias or remove entirely

#### 2.6.4 Commands That May Work With Minimal Changes
- [ ] **`import`** ✅ **UPDATE FOR BOTH SOURCE TYPES**
  - **From private key**: Creates orphaned wallet address (wallet_group_id = NULL, source_type = "private_key")
  - **From mnemonic**: Must specify --account and --wallet-group, creates hierarchical address
  - **STATUS**: Needs parameter updates but core functionality remains

- [ ] **`show`** ✅ **UPDATE FOR HIERARCHY CONTEXT**
  - May need updates to handle both orphaned and hierarchical addresses
  - Should display hierarchy context when showing hierarchical addresses
  - **STATUS**: Needs evaluation and potential parameter updates

#### 2.6.5 Legacy Command Migration Summary
| Current Command | Status | New Command/Action | Priority |
|---|---|---|---|
| `import-multi` | ✅ Remap | Complete 4-level hierarchy creation | High |
| `show-group` | ✅ Remap | `show-wallet-group` with hierarchy display | High |
| `derive` | ❌ Deprecate | `add-address-group` + `add-address` | High |
| `list-groups` | ❌ Deprecate | `list-wallet-groups --account` | High |
| `rename-group` | ❌ Deprecate | `rename-wallet-group --account` | High |
| `list` | ⚠️ Redesign | TBD - comprehensive solution needed | Medium |
| `get` | ⚠️ Redesign | TBD - hierarchy-aware redefinition | Medium |
| `export` | ⚠️ Redesign | TBD - multi-level export options | Medium |
| `delete` | ⚠️ Redesign | Specific `remove-*` commands | Medium |
| `import` | ✅ Update | Parameter updates for hierarchy | Low |
| `show` | ✅ Update | Hierarchy context display | Low |

**Implementation Priority:**
1. **High**: Commands critical for new hierarchy functionality
2. **Medium**: Commands requiring comprehensive redesign (address tomorrow)
3. **Low**: Commands needing minor updates

---

## Phase 3: CLI Argument Structures

### 3.1 New Argument Structures

#### 3.1.1 Master Account Args
- [ ] **`CreateMasterArgs`**
  ```rust
  pub struct CreateMasterArgs {
      #[arg(long)]
      pub account_name: String,
      #[arg(long)]
      pub mnemonic: String,
      #[arg(long)]
      pub passphrase: Option<String>,
  }
  ```

#### 3.1.2 Updated Group Args
- [ ] **Update `ImportMultiArgs`** - Add `account_name: String`
- [ ] **Update `RenameGroupArgs`** - Add `account_name: String`
- [ ] **Update `ShowGroupArgs`** - Add `account_name: String`

#### 3.1.3 New Blockchain Args
- [ ] **`AddBlockchainArgs`** (replaces `DeriveMultiArgs`)
  ```rust
  pub struct AddBlockchainArgs {
      #[arg(long)]
      pub account_name: String,
      #[arg(long)]
      pub group_name: String,
      #[arg(long)]
      pub blockchains: String,
      // REMOVED: mnemonic, account, address_index
  }
  ```

- [ ] **`RemoveBlockchainArgs`**
  ```rust
  pub struct RemoveBlockchainArgs {
      #[arg(long)]
      pub account_name: String,
      #[arg(long)]
      pub group_name: String,
      #[arg(long)]
      pub blockchain: String,
      #[arg(long)]
      pub mnemonic: String,
  }
  ```

#### 3.1.4 New Address Group Args
- [ ] **`AddAddressArgs`**
  ```rust
  pub struct AddAddressArgs {
      #[arg(long)]
      pub account_name: String,
      #[arg(long)]
      pub group_name: String,
      #[arg(long)]
      pub blockchain: String,
      #[arg(long)]
      pub address_name: String,
      #[arg(long)]
      pub label: Option<String>, // Individual address label
  }
  ```

- [ ] **`RenameAddressArgs`**
- [ ] **`RemoveAddressArgs`**
- [ ] **`ListAddressGroupsArgs`**
- [ ] **`ListAddressesArgs`**
  ```rust
  pub struct ListAddressesArgs {
      #[arg(long)]
      pub account_name: String,
      #[arg(long)]
      pub group_name: String,
      #[arg(long)]
      pub blockchain: String,
      #[arg(long)]
      pub address_group: String,
  }
  ```

---

## Phase 4: Auto-Increment & Security Implementation

### 4.1 Auto-Increment Logic
- [ ] Implement account index auto-assignment in `create_wallet_group()`
- [ ] Implement address group index auto-assignment in `create_address_group()`
- [ ] Implement address index auto-assignment in `create_wallet_address()`
- [ ] Add transaction-based increment updates to prevent race conditions

### 4.2 Security Features
- [ ] Implement mnemonic verification for all remove commands
- [ ] Add 'I'm sure' confirmation prompts with exact text matching (single quotes in prompt)
  - Prompt format: "... Type 'I'm sure' to continue."
  - User must type exactly: I'm sure (without quotes)
- [ ] Implement cascade deletion warnings with detailed impact descriptions
- [ ] Add secure mnemonic hash comparison (use SHA-256)

### 4.3 Default Naming System
- [ ] Implement blockchain-specific default address group naming ("btc-0", "eth-0", etc.)
- [ ] Auto-create default address groups when blockchains are added
- [ ] Prevent name conflicts with reserved patterns

---

## Phase 5: Migration & Testing

### 5.1 Data Migration (N/A - New Project)
- [ ] ✅ **SKIP** - No existing data to migrate
- [ ] Verify clean database initialization with new schema

### 5.2 Comprehensive Testing

#### 5.2.1 Database Tests
- [ ] Test all CRUD operations for each table
- [ ] Test auto-increment logic under concurrent access
- [ ] Test cascade deletions work correctly
- [ ] Test foreign key constraints prevent orphaned records
- [ ] Test unique constraints prevent duplicates

#### 5.2.2 CLI Integration Tests
- [ ] Test complete workflow: create-master → create-group → add-blockchain → add-address
- [ ] Test error handling for missing references
- [ ] Test mnemonic verification for remove commands
- [ ] Test "I'm sure" confirmation prompts
- [ ] Test all list commands show correct hierarchical information

#### 5.2.3 Security Tests
- [ ] Test mnemonic verification blocks unauthorized removals
- [ ] Test cascade deletion warnings are accurate
- [ ] Test auto-increment prevents index conflicts
- [ ] Test name uniqueness constraints work properly

---

## Phase 6: Documentation & Finalization

### 6.1 Documentation Updates
- [ ] Update README.md with new 4-level hierarchy explanation
- [ ] Update all command examples to use new syntax
- [ ] Add migration guide from old architecture (if applicable)
- [ ] Update Progress4.json with implementation completion status

### 6.2 Code Cleanup
- [ ] Remove unused/deprecated CLI commands and structs
- [ ] Clean up imports and eliminate warnings
- [ ] Add comprehensive code comments for new hierarchy
- [ ] Update error messages to reflect new structure

---

## Implementation Order & Dependencies

### Critical Path
1. **Database Schema** (Phase 1) - Foundation for everything
2. **Core Database Operations** (Phase 1.3) - Required for CLI commands
3. **Master Account Commands** (Phase 2.1.2) - Entry point to system
4. **Group Management** (Phase 2.2) - Core organizational structure
5. **Blockchain Management** (Phase 2.3) - Core functionality
6. **Address Management** (Phase 2.4) - Complete feature set
7. **Testing** (Phase 5.2) - Ensure reliability

### Parallel Development Opportunities
- CLI argument structures (Phase 3) can be developed alongside database operations
- Security features (Phase 4.2) can be implemented with remove commands
- Documentation (Phase 6.1) can be updated as commands are completed

---

## Success Criteria

### Functional Requirements ✅
- [ ] Single mnemonic supports unlimited wallet groups with independent account indexes
- [ ] Complete auto-increment control prevents index gaps and user errors
- [ ] All blockchain additions work without mnemonic re-entry
- [ ] Remove operations properly cascade with mnemonic verification

### Security Requirements ✅
- [ ] Mnemonic stored once per master account (no duplication)
- [ ] All destructive operations require mnemonic verification + "I'm sure" confirmation
- [ ] Proper cascade deletion warnings prevent accidental data loss

### Usability Requirements ✅
- [ ] Intuitive 4-level hierarchy: Account → Group → AddressGroup → Address
- [ ] Clear naming conventions (btc-0, eth-0 for defaults)
- [ ] Comprehensive list commands for navigation
- [ ] All commands require --account-name for proper context

### Performance Requirements ✅
- [ ] Efficient queries with proper indexing
- [ ] Fast lookups via foreign key relationships
- [ ] No complex joins required for common operations

---

## Risk Assessment & Mitigation

### High Risk Items
1. **Database Schema Complexity** - Multiple foreign key relationships
   - *Mitigation*: Comprehensive testing of all relationships and cascade operations

2. **Auto-Increment Race Conditions** - Concurrent access issues
   - *Mitigation*: Use database transactions for all increment operations

3. **Data Loss from Cascade Deletions** - Accidental removal of large datasets
   - *Mitigation*: Strong confirmation prompts and detailed impact warnings

### Medium Risk Items
1. **CLI Command Complexity** - Many required parameters
   - *Mitigation*: Clear error messages and parameter validation

2. **Migration from Current Structure** - Breaking changes to existing commands
   - *Mitigation*: Comprehensive documentation and migration guides

---

## Estimated Timeline

- **Phase 1 (Database)**: 3-4 days
- **Phase 2 (CLI Commands)**: 4-5 days
- **Phase 3 (Arguments)**: 2-3 days
- **Phase 4 (Security)**: 2-3 days
- **Phase 5 (Testing)**: 3-4 days
- **Phase 6 (Documentation)**: 1-2 days

**Total Estimated Time**: 15-21 days

---

*This implementation plan provides a comprehensive roadmap for migrating to the new 4-level hierarchical architecture while maintaining all existing functionality and adding improved security and usability features.*