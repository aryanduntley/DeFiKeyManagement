use rusqlite::{Connection, params, Result as SqlResult};
use anyhow::{Result, Context, bail};
use chrono::{DateTime, Utc, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sha2::{Sha256, Digest};

// ========== 4-LEVEL HIERARCHICAL STRUCTURES ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterAccount {
    pub id: Option<i64>,
    pub name: String,
    pub mnemonic: String,
    pub master_private_key: String,
    pub passphrase: Option<String>,
    pub next_account_index: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletGroup {
    pub id: Option<i64>,
    pub master_account_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub account_index: u32, // Auto-assigned sequential index
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressGroup {
    pub id: Option<i64>,
    pub wallet_group_id: i64,
    pub blockchain: String,
    pub name: String,
    pub address_group_index: u32, // Auto-assigned sequential index per blockchain
    pub next_address_index: u32, // Track next address index for this group
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletAddress {
    pub id: Option<i64>,
    pub wallet_group_id: Option<i64>, // NULL for private_key-only wallets (orphaned entries)
    pub address_group_id: Option<i64>, // NULL for private_key-only wallets (orphaned entries)
    pub blockchain: String,
    pub address: String,
    pub address_with_checksum: Option<String>,
    pub private_key: String,
    pub public_key: Option<String>,
    pub derivation_path: Option<String>, // NULL for private_key-only wallets (no derivation)
    pub address_index: Option<u32>, // NULL for private_key-only wallets (no derivation sequence)
    pub label: Option<String>, // Individual address label (empty by default)
    pub source_type: String, // "mnemonic" or "private_key"
    pub explorer_url: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub additional_data: HashMap<String, String>, // Blockchain-specific metadata
    pub secondary_addresses: HashMap<String, String>, // Alternative address formats
}

// ========== SUMMARY STRUCTURES ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterAccountSummary {
    pub id: i64,
    pub name: String,
    pub wallet_group_count: i64,
    pub total_addresses: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletGroupSummary {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub account_index: u32,
    pub address_group_count: i64,
    pub total_addresses: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressGroupSummary {
    pub id: i64,
    pub name: String,
    pub blockchain: String,
    pub address_group_index: u32,
    pub address_count: i64,
    pub created_at: DateTime<Utc>,
}

// ========== BULK OPERATION RESULTS ==========

#[derive(Debug, Clone)]
pub struct HierarchyResult {
    pub master_account_id: i64,
    pub wallet_group_id: i64,
    pub account_index: u32,
    pub address_groups: Vec<(i64, String)>, // (address_group_id, blockchain)
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)
            .context("Failed to open database connection")?;

        let db = Database { conn };
        db.create_tables()?;
        Ok(db)
    }

    fn create_tables(&self) -> Result<()> {
        // Level 1: Master Accounts - Root level mnemonic storage
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS master_accounts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL,
                mnemonic TEXT NOT NULL,
                master_private_key TEXT NOT NULL,
                passphrase TEXT,
                next_account_index INTEGER DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            "#,
            [],
        ).context("Failed to create master_accounts table")?;

        // Level 2: Wallet Groups - Named collections under master account
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS wallet_groups (
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
            "#,
            [],
        ).context("Failed to create wallet_groups table")?;

        // Level 3: Address Groups - Named address collections per blockchain per wallet group
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS address_groups (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                wallet_group_id INTEGER NOT NULL,
                blockchain TEXT NOT NULL,
                name TEXT NOT NULL,
                address_group_index INTEGER NOT NULL,
                next_address_index INTEGER DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (wallet_group_id) REFERENCES wallet_groups(id) ON DELETE CASCADE,
                UNIQUE(wallet_group_id, name),
                UNIQUE(wallet_group_id, blockchain, address_group_index)
            );
            "#,
            [],
        ).context("Failed to create address_groups table")?;

        // Level 4: Wallet Addresses - Individual addresses with dual references
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS wallet_addresses (
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
                source_type TEXT NOT NULL DEFAULT 'mnemonic',
                explorer_url TEXT,
                notes TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (wallet_group_id) REFERENCES wallet_groups(id) ON DELETE CASCADE,
                FOREIGN KEY (address_group_id) REFERENCES address_groups(id) ON DELETE CASCADE,
                CHECK (address_group_id IS NULL OR address_index IS NOT NULL)
            );
            "#,
            [],
        ).context("Failed to create wallet_addresses table")?;

        // Preserved: Table for blockchain-specific additional data (links to wallet_addresses.id)
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS wallet_additional_data (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                wallet_id INTEGER NOT NULL,
                data_key TEXT NOT NULL,
                data_value TEXT NOT NULL,
                data_type TEXT DEFAULT 'string',
                FOREIGN KEY (wallet_id) REFERENCES wallet_addresses(id) ON DELETE CASCADE,
                UNIQUE(wallet_id, data_key)
            );
            "#,
            [],
        ).context("Failed to create wallet_additional_data table")?;

        // Preserved: Table for secondary addresses (EVM, legacy, etc.) (links to wallet_addresses.id)
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS wallet_secondary_addresses (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                wallet_id INTEGER NOT NULL,
                address_type TEXT NOT NULL,
                address TEXT NOT NULL,
                address_with_checksum TEXT,
                FOREIGN KEY (wallet_id) REFERENCES wallet_addresses(id) ON DELETE CASCADE,
                UNIQUE(wallet_id, address_type)
            );
            "#,
            [],
        ).context("Failed to create wallet_secondary_addresses table")?;

        // Create indexes for performance - Hierarchical structure optimized

        // Master Accounts indexes
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_master_accounts_name ON master_accounts(name);",
            [],
        ).context("Failed to create master accounts name index")?;

        // Wallet Groups indexes
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_wallet_groups_master_account_id ON wallet_groups(master_account_id);",
            [],
        ).context("Failed to create wallet groups master account id index")?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_wallet_groups_name ON wallet_groups(name);",
            [],
        ).context("Failed to create wallet groups name index")?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_wallet_groups_account_index ON wallet_groups(account_index);",
            [],
        ).context("Failed to create wallet groups account index")?;

        // Address Groups indexes
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_address_groups_wallet_group_id ON address_groups(wallet_group_id);",
            [],
        ).context("Failed to create address groups wallet group id index")?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_address_groups_blockchain ON address_groups(blockchain);",
            [],
        ).context("Failed to create address groups blockchain index")?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_address_groups_name ON address_groups(name);",
            [],
        ).context("Failed to create address groups name index")?;

        // Wallet Addresses indexes
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_wallet_addresses_wallet_group_id ON wallet_addresses(wallet_group_id);",
            [],
        ).context("Failed to create wallet addresses wallet group id index")?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_wallet_addresses_address_group_id ON wallet_addresses(address_group_id);",
            [],
        ).context("Failed to create wallet addresses address group id index")?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_wallet_addresses_blockchain ON wallet_addresses(blockchain);",
            [],
        ).context("Failed to create wallet addresses blockchain index")?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_wallet_addresses_address ON wallet_addresses(address);",
            [],
        ).context("Failed to create wallet addresses address index")?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_wallet_addresses_source_type ON wallet_addresses(source_type);",
            [],
        ).context("Failed to create wallet addresses source type index")?;

        // Preserved metadata indexes
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_wallet_additional_data_wallet_id ON wallet_additional_data(wallet_id);",
            [],
        ).context("Failed to create additional data wallet_id index")?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_wallet_additional_data_key ON wallet_additional_data(data_key);",
            [],
        ).context("Failed to create additional data key index")?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_wallet_secondary_addresses_wallet_id ON wallet_secondary_addresses(wallet_id);",
            [],
        ).context("Failed to create secondary addresses wallet_id index")?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_wallet_secondary_addresses_type ON wallet_secondary_addresses(address_type);",
            [],
        ).context("Failed to create secondary addresses type index")?;

        Ok(())
    }

    // ========== MASTER ACCOUNT OPERATIONS ==========

    /// Creates a new master account with mnemonic storage
    pub fn create_master_account(&self, name: &str, mnemonic: &str, master_private_key: &str, passphrase: Option<&str>) -> Result<i64> {
        let mut stmt = self.conn.prepare(
            "INSERT INTO master_accounts (name, mnemonic, master_private_key, passphrase) VALUES (?1, ?2, ?3, ?4)"
        ).context("Failed to prepare master account insert")?;

        stmt.execute(params![name, mnemonic, master_private_key, passphrase])
            .context("Failed to insert master account")?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Gets master account by name
    pub fn get_master_account_by_name(&self, name: &str) -> Result<Option<MasterAccount>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, mnemonic, master_private_key, passphrase, next_account_index, created_at, updated_at FROM master_accounts WHERE name = ?1"
        ).context("Failed to prepare master account query")?;

        let account_result = stmt.query_row([name], |row| {
            Ok(MasterAccount {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                mnemonic: row.get(2)?,
                master_private_key: row.get(3)?,
                passphrase: row.get(4)?,
                next_account_index: row.get(5)?,
                created_at: self.parse_datetime(&row.get::<_, String>(6)?)?,
                updated_at: self.parse_datetime(&row.get::<_, String>(7)?)?,
            })
        });

        match account_result {
            Ok(account) => Ok(Some(account)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(anyhow::Error::from(e).context("Failed to query master account")),
        }
    }

    /// Lists all master accounts with summary information
    pub fn list_master_accounts(&self) -> Result<Vec<MasterAccountSummary>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                m.id, m.name, m.created_at,
                COUNT(DISTINCT g.id) as wallet_group_count,
                COUNT(DISTINCT a.id) as total_addresses
            FROM master_accounts m
            LEFT JOIN wallet_groups g ON m.id = g.master_account_id
            LEFT JOIN address_groups ag ON g.id = ag.wallet_group_id
            LEFT JOIN wallet_addresses a ON ag.id = a.address_group_id
            GROUP BY m.id, m.name, m.created_at
            ORDER BY m.created_at DESC
            "#
        ).context("Failed to prepare master accounts query")?;

        let account_iter = stmt.query_map([], |row| {
            Ok(MasterAccountSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: self.parse_datetime(&row.get::<_, String>(2)?)?,
                wallet_group_count: row.get(3)?,
                total_addresses: row.get(4)?,
            })
        }).context("Failed to query master accounts")?;

        let mut accounts = Vec::new();
        for account_result in account_iter {
            accounts.push(account_result.context("Failed to parse master account summary")?);
        }

        Ok(accounts)
    }

    /// Deletes a master account and all associated data (requires mnemonic verification)
    pub fn delete_master_account(&self, name: &str, mnemonic_verification: &str) -> Result<bool> {
        // Verify mnemonic first
        if let Some(account) = self.get_master_account_by_name(name)? {
            if account.mnemonic != mnemonic_verification {
                bail!("Mnemonic verification failed. Cannot delete master account.");
            }
        } else {
            return Ok(false); // Account doesn't exist
        }

        let rows_affected = self.conn.execute(
            "DELETE FROM master_accounts WHERE name = ?1",
            params![name],
        ).context("Failed to delete master account")?;

        Ok(rows_affected > 0)
    }

    // ========== WALLET GROUP OPERATIONS ==========

    /// Creates a new wallet group with auto-assigned account index
    pub fn create_wallet_group(&self, master_account_id: i64, name: &str, description: Option<&str>) -> Result<(i64, u32)> {
        let tx = self.conn.unchecked_transaction()?;

        // Get next account index
        let next_account_index: u32 = tx.query_row(
            "SELECT next_account_index FROM master_accounts WHERE id = ?1",
            [master_account_id],
            |row| Ok(row.get(0)?)
        ).context("Failed to get next account index")?;

        // Insert wallet group
        let group_id = {
            let mut stmt = tx.prepare(
                "INSERT INTO wallet_groups (master_account_id, name, description, account_index) VALUES (?1, ?2, ?3, ?4)"
            ).context("Failed to prepare wallet group insert")?;

            stmt.execute(params![master_account_id, name, description, next_account_index])
                .context("Failed to insert wallet group")?;

            tx.last_insert_rowid()
        };

        // Update master account's next_account_index
        tx.execute(
            "UPDATE master_accounts SET next_account_index = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
            params![next_account_index + 1, master_account_id],
        ).context("Failed to update master account next_account_index")?;

        tx.commit().context("Failed to commit wallet group creation")?;
        Ok((group_id, next_account_index))
    }

    /// Gets wallet group by name within a master account
    pub fn get_wallet_group_by_name(&self, master_account_id: i64, name: &str) -> Result<Option<WalletGroup>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, master_account_id, name, description, account_index, created_at, updated_at FROM wallet_groups WHERE master_account_id = ?1 AND name = ?2"
        ).context("Failed to prepare wallet group query")?;

        let group_result = stmt.query_row([&master_account_id.to_string(), name], |row| {
            Ok(WalletGroup {
                id: Some(row.get(0)?),
                master_account_id: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                account_index: row.get(4)?,
                created_at: self.parse_datetime(&row.get::<_, String>(5)?)?,
                updated_at: self.parse_datetime(&row.get::<_, String>(6)?)?,
            })
        });

        match group_result {
            Ok(group) => Ok(Some(group)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(anyhow::Error::from(e).context("Failed to query wallet group")),
        }
    }

    /// Lists wallet groups for a master account with summary information
    pub fn list_wallet_groups(&self, master_account_id: i64) -> Result<Vec<WalletGroupSummary>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                g.id, g.name, g.description, g.account_index, g.created_at,
                COUNT(DISTINCT ag.id) as address_group_count,
                COUNT(DISTINCT a.id) as total_addresses
            FROM wallet_groups g
            LEFT JOIN address_groups ag ON g.id = ag.wallet_group_id
            LEFT JOIN wallet_addresses a ON ag.id = a.address_group_id
            WHERE g.master_account_id = ?1
            GROUP BY g.id, g.name, g.description, g.account_index, g.created_at
            ORDER BY g.account_index
            "#
        ).context("Failed to prepare wallet groups query")?;

        let group_iter = stmt.query_map([master_account_id], |row| {
            Ok(WalletGroupSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                account_index: row.get(3)?,
                address_group_count: row.get(5)?,
                total_addresses: row.get(6)?,
                created_at: self.parse_datetime(&row.get::<_, String>(4)?)?,
            })
        }).context("Failed to query wallet groups")?;

        let mut groups = Vec::new();
        for group_result in group_iter {
            groups.push(group_result.context("Failed to parse wallet group summary")?);
        }

        Ok(groups)
    }

    // Helper method to parse datetime strings (handles both RFC3339 and SQLite formats)
    fn parse_datetime(&self, datetime_str: &str) -> SqlResult<DateTime<Utc>> {
        if datetime_str.contains('T') {
            // RFC3339 format
            Ok(DateTime::parse_from_rfc3339(datetime_str)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
                .with_timezone(&Utc))
        } else {
            // SQLite CURRENT_TIMESTAMP format: "YYYY-MM-DD HH:MM:SS"
            Ok(NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M:%S")
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
                .and_utc())
        }
    }

    // Helper method to load additional data for a wallet address
    fn load_additional_data(&self, wallet_id: i64) -> Result<HashMap<String, String>> {
        let mut stmt = self.conn.prepare(
            "SELECT data_key, data_value FROM wallet_additional_data WHERE wallet_id = ?1"
        ).context("Failed to prepare additional data query")?;

        let data_iter = stmt.query_map([wallet_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        }).context("Failed to query additional data")?;

        let mut data = HashMap::new();
        for item in data_iter {
            let (key, value) = item.context("Failed to parse additional data row")?;
            data.insert(key, value);
        }
        Ok(data)
    }

    // Helper method to load secondary addresses for a wallet address
    fn load_secondary_addresses(&self, wallet_id: i64) -> Result<HashMap<String, String>> {
        let mut stmt = self.conn.prepare(
            "SELECT address_type, address FROM wallet_secondary_addresses WHERE wallet_id = ?1"
        ).context("Failed to prepare secondary addresses query")?;

        let addr_iter = stmt.query_map([wallet_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        }).context("Failed to query secondary addresses")?;

        let mut addresses = HashMap::new();
        for item in addr_iter {
            let (addr_type, address) = item.context("Failed to parse secondary address row")?;
            addresses.insert(addr_type, address);
        }
        Ok(addresses)
    }

    // ========== ADDRESS GROUP OPERATIONS ==========

    /// Creates or gets default address group for a blockchain (e.g., "bitcoin-0", "ethereum-0")
    pub fn get_or_create_default_address_group(&self, wallet_group_id: i64, blockchain: &str) -> Result<i64> {
        let default_name = format!("{}-0", blockchain);

        // Check if default address group exists
        if let Some(group) = self.get_address_group_by_name(wallet_group_id, &default_name)? {
            return Ok(group.id.unwrap());
        }

        // Create new default address group
        self.create_address_group(wallet_group_id, blockchain, &default_name)
    }

    /// Creates a new address group with auto-assigned index
    pub fn create_address_group(&self, wallet_group_id: i64, blockchain: &str, name: &str) -> Result<i64> {
        let tx = self.conn.unchecked_transaction()?;

        // Get next address group index for this blockchain within the wallet group
        let next_index: u32 = tx.query_row(
            "SELECT COALESCE(MAX(address_group_index), -1) + 1 FROM address_groups WHERE wallet_group_id = ?1 AND blockchain = ?2",
            params![wallet_group_id, blockchain],
            |row| Ok(row.get(0)?)
        ).context("Failed to get next address group index")?;

        // Insert address group
        let group_id = {
            let mut stmt = tx.prepare(
                "INSERT INTO address_groups (wallet_group_id, blockchain, name, address_group_index) VALUES (?1, ?2, ?3, ?4)"
            ).context("Failed to prepare address group insert")?;

            stmt.execute(params![wallet_group_id, blockchain, name, next_index])
                .context("Failed to insert address group")?;

            tx.last_insert_rowid()
        };

        tx.commit().context("Failed to commit address group creation")?;
        Ok(group_id)
    }

    /// Gets address group by name within a wallet group
    pub fn get_address_group_by_name(&self, wallet_group_id: i64, name: &str) -> Result<Option<AddressGroup>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, wallet_group_id, blockchain, name, address_group_index, next_address_index, created_at, updated_at FROM address_groups WHERE wallet_group_id = ?1 AND name = ?2"
        ).context("Failed to prepare address group query")?;

        let group_result = stmt.query_row([&wallet_group_id.to_string(), name], |row| {
            Ok(AddressGroup {
                id: Some(row.get(0)?),
                wallet_group_id: row.get(1)?,
                blockchain: row.get(2)?,
                name: row.get(3)?,
                address_group_index: row.get(4)?,
                next_address_index: row.get(5)?,
                created_at: self.parse_datetime(&row.get::<_, String>(6)?)?,
                updated_at: self.parse_datetime(&row.get::<_, String>(7)?)?,
            })
        });

        match group_result {
            Ok(group) => Ok(Some(group)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(anyhow::Error::from(e).context("Failed to query address group")),
        }
    }

    /// Lists address groups for a wallet group, optionally filtered by blockchain
    pub fn list_address_groups(&self, wallet_group_id: i64, blockchain: Option<&str>) -> Result<Vec<AddressGroupSummary>> {
        let (query, params): (String, Vec<rusqlite::types::Value>) = match blockchain {
            Some(chain) => (
                r#"
                SELECT
                    ag.id, ag.name, ag.blockchain, ag.address_group_index, ag.created_at,
                    COUNT(a.id) as address_count
                FROM address_groups ag
                LEFT JOIN wallet_addresses a ON ag.id = a.address_group_id
                WHERE ag.wallet_group_id = ?1 AND ag.blockchain = ?2
                GROUP BY ag.id, ag.name, ag.blockchain, ag.address_group_index, ag.created_at
                ORDER BY ag.blockchain, ag.address_group_index
                "#.to_string(),
                vec![wallet_group_id.into(), chain.to_string().into()]
            ),
            None => (
                r#"
                SELECT
                    ag.id, ag.name, ag.blockchain, ag.address_group_index, ag.created_at,
                    COUNT(a.id) as address_count
                FROM address_groups ag
                LEFT JOIN wallet_addresses a ON ag.id = a.address_group_id
                WHERE ag.wallet_group_id = ?1
                GROUP BY ag.id, ag.name, ag.blockchain, ag.address_group_index, ag.created_at
                ORDER BY ag.blockchain, ag.address_group_index
                "#.to_string(),
                vec![wallet_group_id.into()]
            )
        };

        let mut stmt = self.conn.prepare(&query)
            .context("Failed to prepare address groups query")?;

        let group_iter = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
            Ok(AddressGroupSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                blockchain: row.get(2)?,
                address_group_index: row.get(3)?,
                address_count: row.get(5)?,
                created_at: self.parse_datetime(&row.get::<_, String>(4)?)?,
            })
        }).context("Failed to query address groups")?;

        let mut groups = Vec::new();
        for group_result in group_iter {
            groups.push(group_result.context("Failed to parse address group summary")?);
        }

        Ok(groups)
    }

    // ========== WALLET ADDRESS OPERATIONS ==========

    /// Creates a wallet address within an address group (for mnemonic-derived addresses)
    pub fn create_wallet_address(&self, wallet_address: &WalletAddress) -> Result<i64> {
        let tx = self.conn.unchecked_transaction()?;

        // Get next address index if this is a mnemonic-derived address
        let address_index = if let Some(address_group_id) = wallet_address.address_group_id {
            // Get and increment next_address_index
            let next_index: u32 = tx.query_row(
                "SELECT next_address_index FROM address_groups WHERE id = ?1",
                [address_group_id],
                |row| Ok(row.get(0)?)
            ).context("Failed to get next address index")?;

            // Update next_address_index
            tx.execute(
                "UPDATE address_groups SET next_address_index = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
                params![next_index + 1, address_group_id],
            ).context("Failed to update next address index")?;

            Some(next_index)
        } else {
            // Orphaned address (private_key-only)
            None
        };

        // Insert wallet address
        let wallet_id = {
            let mut stmt = tx.prepare(
                r#"
                INSERT INTO wallet_addresses (
                    wallet_group_id, address_group_id, blockchain, address, address_with_checksum,
                    private_key, public_key, derivation_path, address_index, label,
                    source_type, explorer_url, notes
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                "#
            ).context("Failed to prepare wallet address insert")?;

            stmt.execute(params![
                wallet_address.wallet_group_id,
                wallet_address.address_group_id,
                wallet_address.blockchain,
                wallet_address.address,
                wallet_address.address_with_checksum,
                wallet_address.private_key,
                wallet_address.public_key,
                wallet_address.derivation_path,
                address_index,
                wallet_address.label,
                wallet_address.source_type,
                wallet_address.explorer_url,
                wallet_address.notes,
            ]).context("Failed to insert wallet address")?;

            tx.last_insert_rowid()
        };

        // Insert additional data
        if !wallet_address.additional_data.is_empty() {
            let mut data_stmt = tx.prepare(
                "INSERT INTO wallet_additional_data (wallet_id, data_key, data_value) VALUES (?1, ?2, ?3)"
            ).context("Failed to prepare additional data insert")?;

            for (key, value) in &wallet_address.additional_data {
                data_stmt.execute(params![wallet_id, key, value])
                    .context("Failed to insert additional data")?;
            }
        }

        // Insert secondary addresses
        if !wallet_address.secondary_addresses.is_empty() {
            let mut addr_stmt = tx.prepare(
                "INSERT INTO wallet_secondary_addresses (wallet_id, address_type, address) VALUES (?1, ?2, ?3)"
            ).context("Failed to prepare secondary address insert")?;

            for (addr_type, address) in &wallet_address.secondary_addresses {
                addr_stmt.execute(params![wallet_id, addr_type, address])
                    .context("Failed to insert secondary address")?;
            }
        }

        tx.commit().context("Failed to commit wallet address creation")?;
        Ok(wallet_id)
    }
    /// Creates an orphaned wallet address (for private_key-only addresses)
    pub fn create_orphaned_wallet_address(&self, wallet_address: &WalletAddress) -> Result<i64> {
        // Ensure this is marked as orphaned
        let mut orphaned_address = wallet_address.clone();
        orphaned_address.wallet_group_id = None;
        orphaned_address.address_group_id = None;
        orphaned_address.derivation_path = None;
        orphaned_address.address_index = None;
        orphaned_address.source_type = "private_key".to_string();

        self.create_wallet_address(&orphaned_address)
    }

    /// Gets wallet addresses for a specific address group
    pub fn get_wallet_addresses_by_address_group(&self, address_group_id: i64) -> Result<Vec<WalletAddress>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, wallet_group_id, address_group_id, blockchain, address, address_with_checksum,
                   private_key, public_key, derivation_path, address_index, label,
                   source_type, explorer_url, notes, created_at
            FROM wallet_addresses
            WHERE address_group_id = ?1
            ORDER BY address_index
            "#
        ).context("Failed to prepare wallet addresses query")?;

        let address_iter = stmt.query_map([address_group_id], |row| {
            self.build_wallet_address_from_row(row)
        }).context("Failed to query wallet addresses")?;

        let mut addresses = Vec::new();
        for address_result in address_iter {
            let address = address_result.context("Failed to parse wallet address")?;
            let completed_address = self.complete_wallet_address(address)?;
            addresses.push(completed_address);
        }

        Ok(addresses)
    }

    /// Gets all orphaned wallet addresses (private_key-only)
    pub fn get_orphaned_wallet_addresses(&self) -> Result<Vec<WalletAddress>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, wallet_group_id, address_group_id, blockchain, address, address_with_checksum,
                   private_key, public_key, derivation_path, address_index, label,
                   source_type, explorer_url, notes, created_at
            FROM wallet_addresses
            WHERE wallet_group_id IS NULL AND address_group_id IS NULL
            ORDER BY created_at DESC
            "#
        ).context("Failed to prepare orphaned addresses query")?;

        let address_iter = stmt.query_map([], |row| {
            self.build_wallet_address_from_row(row)
        }).context("Failed to query orphaned addresses")?;

        let mut addresses = Vec::new();
        for address_result in address_iter {
            let address = address_result.context("Failed to parse wallet address")?;
            let completed_address = self.complete_wallet_address(address)?;
            addresses.push(completed_address);
        }

        Ok(addresses)
    }

    // Helper method to build WalletAddress from database row
    fn build_wallet_address_from_row(&self, row: &rusqlite::Row) -> SqlResult<WalletAddress> {
        Ok(WalletAddress {
            id: Some(row.get(0)?),
            wallet_group_id: row.get(1)?,
            address_group_id: row.get(2)?,
            blockchain: row.get(3)?,
            address: row.get(4)?,
            address_with_checksum: row.get(5)?,
            private_key: row.get(6)?,
            public_key: row.get(7)?,
            derivation_path: row.get(8)?,
            address_index: row.get(9)?,
            label: row.get(10)?,
            source_type: row.get(11)?,
            explorer_url: row.get(12)?,
            notes: row.get(13)?,
            created_at: self.parse_datetime(&row.get::<_, String>(14)?).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
            additional_data: HashMap::new(),
            secondary_addresses: HashMap::new(),
        })
    }

    // Helper method to complete wallet address with additional data
    fn complete_wallet_address(&self, mut address: WalletAddress) -> Result<WalletAddress> {
        if let Some(address_id) = address.id {
            address.additional_data = self.load_additional_data(address_id)?;
            address.secondary_addresses = self.load_secondary_addresses(address_id)?;
        }
        Ok(address)
    }
    /// Gets wallet address by address string
    pub fn get_wallet_address_by_address(&self, address: &str) -> Result<Option<WalletAddress>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, wallet_group_id, address_group_id, blockchain, address, address_with_checksum,
                   private_key, public_key, derivation_path, address_index, label,
                   source_type, explorer_url, notes, created_at
            FROM wallet_addresses WHERE address = ?1
            "#
        ).context("Failed to prepare wallet address query")?;

        let mut rows = stmt.query_map([address], |row| {
            self.build_wallet_address_from_row(row)
        }).context("Failed to query wallet address")?;

        match rows.next() {
            Some(address_result) => {
                let address = address_result.context("Failed to parse wallet address")?;
                let completed_address = self.complete_wallet_address(address)?;
                Ok(Some(completed_address))
            },
            None => Ok(None),
        }
    }
    /// Gets wallet address by label
    pub fn get_wallet_address_by_label(&self, label: &str) -> Result<Option<WalletAddress>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, wallet_group_id, address_group_id, blockchain, address, address_with_checksum,
                   private_key, public_key, derivation_path, address_index, label,
                   source_type, explorer_url, notes, created_at
            FROM wallet_addresses WHERE label = ?1
            "#
        ).context("Failed to prepare wallet address query")?;

        let mut rows = stmt.query_map([label], |row| {
            self.build_wallet_address_from_row(row)
        }).context("Failed to query wallet address by label")?;

        match rows.next() {
            Some(address_result) => {
                let address = address_result.context("Failed to parse wallet address")?;
                let completed_address = self.complete_wallet_address(address)?;
                Ok(Some(completed_address))
            },
            None => Ok(None),
        }
    }
    // ========== BULK OPERATIONS ==========

    /// Creates complete hierarchy from mnemonic for import-multi command
    pub fn create_complete_hierarchy_from_mnemonic(
        &self,
        account_name: &str,
        wallet_group_name: &str,
        blockchain_list: &[String],
        mnemonic: &str,
        master_private_key: &str,
        passphrase: Option<&str>,
        description: Option<&str>
    ) -> Result<HierarchyResult> {
        let tx = self.conn.unchecked_transaction()?;

        // Create or get master account
        let master_account_id = match self.get_master_account_by_name(account_name)? {
            Some(existing_account) => {
                if existing_account.mnemonic != mnemonic {
                    bail!("Master account '{}' already exists with a different mnemonic.", account_name);
                }
                existing_account.id.unwrap()
            },
            None => {
                self.create_master_account(account_name, mnemonic, master_private_key, passphrase)?
            }
        };

        // Create wallet group
        let (wallet_group_id, account_index) = self.create_wallet_group(master_account_id, wallet_group_name, description)?;

        // Create default address groups and collect results
        let mut address_groups = Vec::new();
        for blockchain in blockchain_list {
            let address_group_id = self.get_or_create_default_address_group(wallet_group_id, blockchain)?;
            address_groups.push((address_group_id, blockchain.clone()));
        }

        tx.commit().context("Failed to commit hierarchy creation")?;

        Ok(HierarchyResult {
            master_account_id,
            wallet_group_id,
            account_index,
            address_groups,
        })
    }

    // ========== UTILITY & MANAGEMENT OPERATIONS ==========

    /// Updates wallet address label
    pub fn update_wallet_address_label(&self, address: &str, new_label: &str) -> Result<bool> {
        let rows_affected = self.conn.execute(
            "UPDATE wallet_addresses SET label = ?1 WHERE address = ?2",
            params![new_label, address],
        ).context("Failed to update wallet address label")?;

        Ok(rows_affected > 0)
    }

    /// Renames wallet group within a master account
    pub fn rename_wallet_group(&self, master_account_id: i64, old_name: &str, new_name: &str) -> Result<bool> {
        // Check if new name already exists within this master account
        if self.get_wallet_group_by_name(master_account_id, new_name)?.is_some() {
            bail!("Wallet group '{}' already exists in this master account.", new_name);
        }

        let rows_affected = self.conn.execute(
            "UPDATE wallet_groups SET name = ?1, updated_at = CURRENT_TIMESTAMP WHERE master_account_id = ?2 AND name = ?3",
            params![new_name, master_account_id, old_name],
        ).context("Failed to rename wallet group")?;

        Ok(rows_affected > 0)
    }

    /// Renames address group within a wallet group
    pub fn rename_address_group(&self, wallet_group_id: i64, old_name: &str, new_name: &str) -> Result<bool> {
        // Check if new name already exists within this wallet group
        if self.get_address_group_by_name(wallet_group_id, new_name)?.is_some() {
            bail!("Address group '{}' already exists in this wallet group.", new_name);
        }

        let rows_affected = self.conn.execute(
            "UPDATE address_groups SET name = ?1, updated_at = CURRENT_TIMESTAMP WHERE wallet_group_id = ?2 AND name = ?3",
            params![new_name, wallet_group_id, old_name],
        ).context("Failed to rename address group")?;

        Ok(rows_affected > 0)
    }

    /// Deletes wallet group and all associated data (requires mnemonic verification)
    pub fn delete_wallet_group(&self, master_account_id: i64, group_name: &str, mnemonic_verification: &str) -> Result<bool> {
        // Verify mnemonic first
        if let Some(master_account) = self.get_master_account_by_name(&format!("{}", master_account_id))? {
            if master_account.mnemonic != mnemonic_verification {
                bail!("Mnemonic verification failed. Cannot delete wallet group.");
            }
        } else {
            return Ok(false);
        }

        let rows_affected = self.conn.execute(
            "DELETE FROM wallet_groups WHERE master_account_id = ?1 AND name = ?2",
            params![master_account_id, group_name],
        ).context("Failed to delete wallet group")?;

        Ok(rows_affected > 0)
    }

    /// Deletes address group and all associated addresses (requires mnemonic verification)
    pub fn delete_address_group(&self, wallet_group_id: i64, address_group_name: &str, mnemonic_verification: &str) -> Result<bool> {
        // Get master account for mnemonic verification
        let master_account_mnemonic: String = self.conn.query_row(
            "SELECT ma.mnemonic FROM master_accounts ma JOIN wallet_groups wg ON ma.id = wg.master_account_id WHERE wg.id = ?1",
            [wallet_group_id],
            |row| Ok(row.get(0)?)
        ).context("Failed to get master account mnemonic")?;

        if master_account_mnemonic != mnemonic_verification {
            bail!("Mnemonic verification failed. Cannot delete address group.");
        }

        let rows_affected = self.conn.execute(
            "DELETE FROM address_groups WHERE wallet_group_id = ?1 AND name = ?2",
            params![wallet_group_id, address_group_name],
        ).context("Failed to delete address group")?;

        Ok(rows_affected > 0)
    }

    /// Deletes individual wallet address (requires mnemonic verification for hierarchical addresses)
    pub fn delete_wallet_address(&self, address: &str, mnemonic_verification: Option<&str>) -> Result<bool> {
        // Check if this is an orphaned address or hierarchical address
        let address_info: (Option<i64>, Option<i64>, String) = self.conn.query_row(
            "SELECT wallet_group_id, address_group_id, source_type FROM wallet_addresses WHERE address = ?1",
            [address],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        ).context("Failed to get address info")?;

        let (wallet_group_id, _address_group_id, source_type) = address_info;

        // If hierarchical address, verify mnemonic
        if wallet_group_id.is_some() && source_type == "mnemonic" {
            if let Some(mnemonic_verify) = mnemonic_verification {
                let master_account_mnemonic: String = self.conn.query_row(
                    "SELECT ma.mnemonic FROM master_accounts ma JOIN wallet_groups wg ON ma.id = wg.master_account_id WHERE wg.id = ?1",
                    [wallet_group_id],
                    |row| Ok(row.get(0)?)
                ).context("Failed to get master account mnemonic")?;

                if master_account_mnemonic != mnemonic_verify {
                    bail!("Mnemonic verification failed. Cannot delete wallet address.");
                }
            } else {
                bail!("Mnemonic verification required for deleting hierarchical addresses.");
            }
        }

        let rows_affected = self.conn.execute(
            "DELETE FROM wallet_addresses WHERE address = ?1",
            params![address],
        ).context("Failed to delete wallet address")?;

        Ok(rows_affected > 0)
    }
    /// Search wallet addresses by term, optionally filtered by blockchain
    pub fn search_wallet_addresses(&self, term: &str, blockchain: Option<&str>) -> Result<Vec<WalletAddress>> {
        let (query, params): (String, Vec<String>) = match blockchain {
            Some(chain) => (
                r#"
                SELECT id, wallet_group_id, address_group_id, blockchain, address, address_with_checksum,
                       private_key, public_key, derivation_path, address_index, label,
                       source_type, explorer_url, notes, created_at
                FROM wallet_addresses
                WHERE blockchain = ?1 AND (
                    label LIKE ?2 OR
                    address LIKE ?2 OR
                    notes LIKE ?2
                )
                ORDER BY created_at DESC
                "#.to_string(),
                vec![chain.to_string(), format!("%{}%", term)]
            ),
            None => (
                r#"
                SELECT id, wallet_group_id, address_group_id, blockchain, address, address_with_checksum,
                       private_key, public_key, derivation_path, address_index, label,
                       source_type, explorer_url, notes, created_at
                FROM wallet_addresses
                WHERE label LIKE ?1 OR address LIKE ?1 OR blockchain LIKE ?1 OR notes LIKE ?1
                ORDER BY created_at DESC
                "#.to_string(),
                vec![format!("%{}%", term)]
            )
        };

        let mut stmt = self.conn.prepare(&query)
            .context("Failed to prepare search statement")?;

        let address_iter = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
            self.build_wallet_address_from_row(row)
        }).context("Failed to search wallet addresses")?;

        let mut addresses = Vec::new();
        for address_result in address_iter {
            let address = address_result.context("Failed to parse wallet address")?;
            let completed_address = self.complete_wallet_address(address)?;
            addresses.push(completed_address);
        }

        Ok(addresses)
    }

}