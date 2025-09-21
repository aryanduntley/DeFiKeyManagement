use rusqlite::{Connection, params, Result as SqlResult};
use anyhow::{Result, Context, bail};
use chrono::{DateTime, Utc, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sha2::Digest;

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
    pub base_wallet_id: i64, // References the parent wallet (child private key)
    pub blockchain: String,
    pub name: String,
    pub address_group_index: u32, // Auto-assigned sequential index per blockchain
    pub next_address_index: u32, // Track next address index for this group
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub id: Option<i64>,
    pub wallet_group_id: Option<i64>, // NULL for standalone wallets (private_key-only)
    pub address_group_id: Option<i64>, // NULL for direct wallets under wallet_group
    pub blockchain: String,
    pub address: String,
    pub address_with_checksum: Option<String>,
    pub private_key: String,
    pub public_key: Option<String>,
    pub derivation_path: Option<String>, // Contains full path like "m/0" or "m/0/5", NULL for standalone wallets
    pub label: Option<String>, // Individual wallet label (empty by default)
    pub source_type: String, // "mnemonic", "master_private_key", or "private_key"
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
    pub total_wallets: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletGroupSummary {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub account_index: u32,
    pub address_group_count: i64,
    pub total_wallets: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressGroupSummary {
    pub id: i64,
    pub name: String,
    pub blockchain: String,
    pub address_group_index: u32,
    pub wallet_count: i64,
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
                base_wallet_id INTEGER NOT NULL,
                blockchain TEXT NOT NULL,
                name TEXT NOT NULL,
                address_group_index INTEGER NOT NULL,
                next_address_index INTEGER DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (wallet_group_id) REFERENCES wallet_groups(id) ON DELETE CASCADE,
                FOREIGN KEY (base_wallet_id) REFERENCES wallets(id) ON DELETE CASCADE,
                UNIQUE(base_wallet_id, name),
                UNIQUE(base_wallet_id, address_group_index)
            );
            "#,
            [],
        ).context("Failed to create address_groups table")?;

        // Level 4: Wallets - Individual wallets with dual references for hierarchy
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS wallets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                wallet_group_id INTEGER, -- NULL for standalone wallets (private_key-only)
                address_group_id INTEGER, -- NULL for direct wallets under wallet_group
                blockchain TEXT NOT NULL,
                address TEXT UNIQUE NOT NULL,
                address_with_checksum TEXT,
                private_key TEXT NOT NULL,
                public_key TEXT,
                derivation_path TEXT, -- Contains full path like "m/0" or "m/0/5", NULL for standalone wallets
                label TEXT, -- Individual wallet label (empty by default)
                source_type TEXT NOT NULL DEFAULT 'mnemonic',
                explorer_url TEXT,
                notes TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (wallet_group_id) REFERENCES wallet_groups(id) ON DELETE CASCADE,
                FOREIGN KEY (address_group_id) REFERENCES address_groups(id) ON DELETE CASCADE
            );
            "#,
            [],
        ).context("Failed to create wallets table")?;

        // Preserved: Table for blockchain-specific additional data (links to wallets.id)
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS wallet_additional_data (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                wallet_id INTEGER NOT NULL,
                data_key TEXT NOT NULL,
                data_value TEXT NOT NULL,
                data_type TEXT DEFAULT 'string',
                FOREIGN KEY (wallet_id) REFERENCES wallets(id) ON DELETE CASCADE,
                UNIQUE(wallet_id, data_key)
            );
            "#,
            [],
        ).context("Failed to create wallet_additional_data table")?;

        // Preserved: Table for secondary addresses (EVM, legacy, etc.) (links to wallets.id)
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS wallet_secondary_addresses (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                wallet_id INTEGER NOT NULL,
                address_type TEXT NOT NULL,
                address TEXT NOT NULL,
                address_with_checksum TEXT,
                FOREIGN KEY (wallet_id) REFERENCES wallets(id) ON DELETE CASCADE,
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

        // Wallets indexes
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_wallets_wallet_group_id ON wallets(wallet_group_id);",
            [],
        ).context("Failed to create wallets wallet group id index")?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_wallets_address_group_id ON wallets(address_group_id);",
            [],
        ).context("Failed to create wallets address group id index")?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_wallets_blockchain ON wallets(blockchain);",
            [],
        ).context("Failed to create wallets blockchain index")?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_wallets_address ON wallets(address);",
            [],
        ).context("Failed to create wallets address index")?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_wallets_source_type ON wallets(source_type);",
            [],
        ).context("Failed to create wallets source type index")?;

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
                COUNT(DISTINCT w.id) as total_wallets
            FROM master_accounts m
            LEFT JOIN wallet_groups g ON m.id = g.master_account_id
            LEFT JOIN address_groups ag ON g.id = ag.wallet_group_id
            LEFT JOIN wallets w ON (g.id = w.wallet_group_id OR ag.id = w.address_group_id)
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
                total_wallets: row.get(4)?,
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
                COUNT(DISTINCT w.id) as total_wallets
            FROM wallet_groups g
            LEFT JOIN address_groups ag ON g.id = ag.wallet_group_id
            LEFT JOIN wallets w ON (g.id = w.wallet_group_id OR ag.id = w.address_group_id)
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
                total_wallets: row.get(6)?,
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
        // This method is deprecated with the new schema - address groups belong to specific wallets
        // For backward compatibility during migration, return an error with guidance
        Err(anyhow::anyhow!("get_or_create_default_address_group is deprecated. Use get_or_create_default_address_group_for_wallet instead."))
    }

    /// Creates or gets default address group for a specific wallet (NEW METHOD)
    pub fn get_or_create_default_address_group_for_wallet(&self, wallet_group_id: i64, base_wallet_id: i64, blockchain: &str) -> Result<i64> {
        let default_name = format!("{}-0", blockchain);

        // Check if default address group exists for this wallet
        if let Some(group) = self.get_address_group_by_name_for_wallet(base_wallet_id, &default_name)? {
            return Ok(group.id.unwrap());
        }

        // Create new default address group for this specific wallet
        self.create_address_group(wallet_group_id, base_wallet_id, blockchain, &default_name)
    }

    /// Creates a new address group with auto-assigned index
    pub fn create_address_group(&self, wallet_group_id: i64, base_wallet_id: i64, blockchain: &str, name: &str) -> Result<i64> {
        let tx = self.conn.unchecked_transaction()?;

        // Get next address group index for this base wallet
        let next_index: u32 = tx.query_row(
            "SELECT COALESCE(MAX(address_group_index), -1) + 1 FROM address_groups WHERE base_wallet_id = ?1",
            params![base_wallet_id],
            |row| Ok(row.get(0)?)
        ).context("Failed to get next address group index")?;

        // Insert address group
        let group_id = {
            let mut stmt = tx.prepare(
                "INSERT INTO address_groups (wallet_group_id, base_wallet_id, blockchain, name, address_group_index) VALUES (?1, ?2, ?3, ?4, ?5)"
            ).context("Failed to prepare address group insert")?;

            stmt.execute(params![wallet_group_id, base_wallet_id, blockchain, name, next_index])
                .context("Failed to insert address group")?;

            tx.last_insert_rowid()
        };

        tx.commit().context("Failed to commit address group creation")?;
        Ok(group_id)
    }

    /// Gets address group by name within a wallet group
    pub fn get_address_group_by_name(&self, wallet_group_id: i64, name: &str) -> Result<Option<AddressGroup>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, wallet_group_id, base_wallet_id, blockchain, name, address_group_index, next_address_index, created_at, updated_at FROM address_groups WHERE wallet_group_id = ?1 AND name = ?2"
        ).context("Failed to prepare address group query")?;

        let group_result = stmt.query_row([&wallet_group_id.to_string(), name], |row| {
            Ok(AddressGroup {
                id: Some(row.get(0)?),
                wallet_group_id: row.get(1)?,
                base_wallet_id: row.get(2)?,
                blockchain: row.get(3)?,
                name: row.get(4)?,
                address_group_index: row.get(5)?,
                next_address_index: row.get(6)?,
                created_at: self.parse_datetime(&row.get::<_, String>(7)?)?,
                updated_at: self.parse_datetime(&row.get::<_, String>(8)?)?,
            })
        });

        match group_result {
            Ok(group) => Ok(Some(group)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(anyhow::Error::from(e).context("Failed to query address group")),
        }
    }

    /// Gets address group by name for a specific wallet (base wallet)
    pub fn get_address_group_by_name_for_wallet(&self, base_wallet_id: i64, name: &str) -> Result<Option<AddressGroup>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, wallet_group_id, base_wallet_id, blockchain, name, address_group_index, next_address_index, created_at, updated_at FROM address_groups WHERE base_wallet_id = ?1 AND name = ?2"
        ).context("Failed to prepare address group query")?;

        let group_result = stmt.query_row([&base_wallet_id.to_string(), name], |row| {
            Ok(AddressGroup {
                id: Some(row.get(0)?),
                wallet_group_id: row.get(1)?,
                base_wallet_id: row.get(2)?,
                blockchain: row.get(3)?,
                name: row.get(4)?,
                address_group_index: row.get(5)?,
                next_address_index: row.get(6)?,
                created_at: self.parse_datetime(&row.get::<_, String>(7)?)?,
                updated_at: self.parse_datetime(&row.get::<_, String>(8)?)?,
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
                    COUNT(w.id) as wallet_count
                FROM address_groups ag
                LEFT JOIN wallets w ON ag.id = w.address_group_id
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
                    COUNT(w.id) as wallet_count
                FROM address_groups ag
                LEFT JOIN wallets w ON ag.id = w.address_group_id
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
                wallet_count: row.get(5)?,
                created_at: self.parse_datetime(&row.get::<_, String>(4)?)?,
            })
        }).context("Failed to query address groups")?;

        let mut groups = Vec::new();
        for group_result in group_iter {
            groups.push(group_result.context("Failed to parse address group summary")?);
        }

        Ok(groups)
    }

    /// Lists address groups for a specific wallet (base wallet)
    pub fn list_address_groups_for_wallet(&self, base_wallet_id: i64) -> Result<Vec<AddressGroupSummary>> {
        let query = r#"
            SELECT
                ag.id, ag.name, ag.blockchain, ag.address_group_index, ag.created_at,
                COUNT(w.id) as wallet_count
            FROM address_groups ag
            LEFT JOIN wallets w ON ag.id = w.address_group_id
            WHERE ag.base_wallet_id = ?1
            GROUP BY ag.id, ag.name, ag.blockchain, ag.address_group_index, ag.created_at
            ORDER BY ag.address_group_index
        "#;

        let mut stmt = self.conn.prepare(query).context("Failed to prepare address groups query")?;
        let group_iter = stmt.query_map([base_wallet_id], |row| {
            Ok(AddressGroupSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                blockchain: row.get(2)?,
                address_group_index: row.get(3)?,
                wallet_count: row.get(5)?,
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

    /// Creates a wallet within hierarchy (for mnemonic-derived wallets)
    pub fn create_wallet(&self, wallet: &Wallet) -> Result<i64> {
        let tx = self.conn.unchecked_transaction()?;

        // Insert wallet - no address_index management needed (handled by derivation_path)
        let wallet_id = {
            let mut stmt = tx.prepare(
                r#"
                INSERT INTO wallets (
                    wallet_group_id, address_group_id, blockchain, address, address_with_checksum,
                    private_key, public_key, derivation_path, label,
                    source_type, explorer_url, notes
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                "#
            ).context("Failed to prepare wallet insert")?;

            stmt.execute(params![
                wallet.wallet_group_id,
                wallet.address_group_id,
                wallet.blockchain,
                wallet.address,
                wallet.address_with_checksum,
                wallet.private_key,
                wallet.public_key,
                wallet.derivation_path,
                wallet.label,
                wallet.source_type,
                wallet.explorer_url,
                wallet.notes,
            ]).context("Failed to insert wallet")?;

            tx.last_insert_rowid()
        };

        // Insert additional data
        if !wallet.additional_data.is_empty() {
            let mut data_stmt = tx.prepare(
                "INSERT INTO wallet_additional_data (wallet_id, data_key, data_value) VALUES (?1, ?2, ?3)"
            ).context("Failed to prepare additional data insert")?;

            for (key, value) in &wallet.additional_data {
                data_stmt.execute(params![wallet_id, key, value])
                    .context("Failed to insert additional data")?;
            }
        }

        // Insert secondary addresses
        if !wallet.secondary_addresses.is_empty() {
            let mut addr_stmt = tx.prepare(
                "INSERT INTO wallet_secondary_addresses (wallet_id, address_type, address) VALUES (?1, ?2, ?3)"
            ).context("Failed to prepare secondary address insert")?;

            for (addr_type, address) in &wallet.secondary_addresses {
                addr_stmt.execute(params![wallet_id, addr_type, address])
                    .context("Failed to insert secondary address")?;
            }
        }

        tx.commit().context("Failed to commit wallet creation")?;
        Ok(wallet_id)
    }

    /// Creates a standalone wallet (for private_key-only wallets)
    pub fn create_standalone_wallet(&self, wallet: &Wallet) -> Result<i64> {
        // Ensure this is marked as standalone
        let mut standalone_wallet = wallet.clone();
        standalone_wallet.wallet_group_id = None;
        standalone_wallet.address_group_id = None;
        standalone_wallet.derivation_path = None;
        standalone_wallet.source_type = "private_key".to_string();

        self.create_wallet(&standalone_wallet)
    }

    /// Gets wallets for a specific address group (subwallets)
    pub fn get_wallets_by_address_group(&self, address_group_id: i64) -> Result<Vec<Wallet>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, wallet_group_id, address_group_id, blockchain, address, address_with_checksum,
                   private_key, public_key, derivation_path, label,
                   source_type, explorer_url, notes, created_at
            FROM wallets
            WHERE address_group_id = ?1
            ORDER BY derivation_path
            "#
        ).context("Failed to prepare wallets query")?;

        let wallet_iter = stmt.query_map([address_group_id], |row| {
            self.build_wallet_from_row(row)
        }).context("Failed to query wallets")?;

        let mut wallets = Vec::new();
        for wallet_result in wallet_iter {
            let wallet = wallet_result.context("Failed to parse wallet")?;
            let completed_wallet = self.complete_wallet(wallet)?;
            wallets.push(completed_wallet);
        }

        Ok(wallets)
    }

    /// Gets all base wallets for a wallet group (address_group_id IS NULL)
    /// These are the child private keys that belong directly to the wallet group
    pub fn get_wallets_by_wallet_group(&self, wallet_group_id: i64) -> Result<Vec<Wallet>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, wallet_group_id, address_group_id, blockchain, address, address_with_checksum,
                   private_key, public_key, derivation_path, label,
                   source_type, explorer_url, notes, created_at
            FROM wallets
            WHERE wallet_group_id = ?1 AND address_group_id IS NULL
            ORDER BY created_at DESC
            "#
        ).context("Failed to prepare wallet group wallets query")?;

        let wallet_iter = stmt.query_map([wallet_group_id], |row| {
            self.build_wallet_from_row(row)
        }).context("Failed to query wallet group wallets")?;

        let mut wallets = Vec::new();
        for wallet_result in wallet_iter {
            let wallet = wallet_result.context("Failed to parse wallet")?;
            let completed_wallet = self.complete_wallet(wallet)?;
            wallets.push(completed_wallet);
        }

        Ok(wallets)
    }

    /// Gets all standalone wallets (private_key-only)
    pub fn get_standalone_wallets(&self) -> Result<Vec<Wallet>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, wallet_group_id, address_group_id, blockchain, address, address_with_checksum,
                   private_key, public_key, derivation_path, label,
                   source_type, explorer_url, notes, created_at
            FROM wallets
            WHERE wallet_group_id IS NULL AND address_group_id IS NULL
            ORDER BY created_at DESC
            "#
        ).context("Failed to prepare standalone wallets query")?;

        let wallet_iter = stmt.query_map([], |row| {
            self.build_wallet_from_row(row)
        }).context("Failed to query standalone wallets")?;

        let mut wallets = Vec::new();
        for wallet_result in wallet_iter {
            let wallet = wallet_result.context("Failed to parse wallet")?;
            let completed_wallet = self.complete_wallet(wallet)?;
            wallets.push(completed_wallet);
        }

        Ok(wallets)
    }

    // Helper method to build Wallet from database row (new schema without address_index)
    fn build_wallet_from_row(&self, row: &rusqlite::Row) -> SqlResult<Wallet> {
        Ok(Wallet {
            id: Some(row.get(0)?),
            wallet_group_id: row.get(1)?,
            address_group_id: row.get(2)?,
            blockchain: row.get(3)?,
            address: row.get(4)?,
            address_with_checksum: row.get(5)?,
            private_key: row.get(6)?,
            public_key: row.get(7)?,
            derivation_path: row.get(8)?,
            label: row.get(9)?,
            source_type: row.get(10)?,
            explorer_url: row.get(11)?,
            notes: row.get(12)?,
            created_at: self.parse_datetime(&row.get::<_, String>(13)?).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
            additional_data: HashMap::new(),
            secondary_addresses: HashMap::new(),
        })
    }

    // Helper method to complete wallet with additional data
    fn complete_wallet(&self, mut wallet: Wallet) -> Result<Wallet> {
        if let Some(wallet_id) = wallet.id {
            wallet.additional_data = self.load_additional_data(wallet_id)?;
            wallet.secondary_addresses = self.load_secondary_addresses(wallet_id)?;
        }
        Ok(wallet)
    }

    /// Gets wallet by address string
    pub fn get_wallet_by_address(&self, address: &str) -> Result<Option<Wallet>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, wallet_group_id, address_group_id, blockchain, address, address_with_checksum,
                   private_key, public_key, derivation_path, label,
                   source_type, explorer_url, notes, created_at
            FROM wallets WHERE address = ?1
            "#
        ).context("Failed to prepare wallet address query")?;

        let mut rows = stmt.query_map([address], |row| {
            self.build_wallet_from_row(row)
        }).context("Failed to query wallet")?;

        match rows.next() {
            Some(wallet_result) => {
                let wallet = wallet_result.context("Failed to parse wallet")?;
                let completed_wallet = self.complete_wallet(wallet)?;
                Ok(Some(completed_wallet))
            },
            None => Ok(None),
        }
    }
    /// Gets wallet by label
    pub fn get_wallet_by_label(&self, label: &str) -> Result<Option<Wallet>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, wallet_group_id, address_group_id, blockchain, address, address_with_checksum,
                   private_key, public_key, derivation_path, label,
                   source_type, explorer_url, notes, created_at
            FROM wallets WHERE label = ?1
            "#
        ).context("Failed to prepare wallet address query")?;

        let mut rows = stmt.query_map([label], |row| {
            self.build_wallet_from_row(row)
        }).context("Failed to query wallet by label")?;

        match rows.next() {
            Some(wallet_result) => {
                let wallet = wallet_result.context("Failed to parse wallet")?;
                let completed_wallet = self.complete_wallet(wallet)?;
                Ok(Some(completed_wallet))
            },
            None => Ok(None),
        }
    }

    /// Gets wallet by label within a specific wallet group (for add-address-group command)
    pub fn get_wallet_by_name_in_group(&self, wallet_group_id: i64, wallet_label: &str) -> Result<Option<Wallet>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, wallet_group_id, address_group_id, blockchain, address, address_with_checksum,
                   private_key, public_key, derivation_path, label,
                   source_type, explorer_url, notes, created_at
            FROM wallets WHERE wallet_group_id = ?1 AND label = ?2 AND address_group_id IS NULL
            "#
        ).context("Failed to prepare wallet query")?;

        let mut rows = stmt.query_map([&wallet_group_id.to_string(), wallet_label], |row| {
            self.build_wallet_from_row(row)
        }).context("Failed to query wallet by label in group")?;

        match rows.next() {
            Some(wallet_result) => {
                let wallet = wallet_result.context("Failed to parse wallet")?;
                let completed_wallet = self.complete_wallet(wallet)?;
                Ok(Some(completed_wallet))
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

    /// Updates wallet label
    pub fn update_wallet_label(&self, address: &str, new_label: &str) -> Result<bool> {
        let rows_affected = self.conn.execute(
            "UPDATE wallets SET label = ?1 WHERE address = ?2",
            params![new_label, address],
        ).context("Failed to update wallet label")?;

        Ok(rows_affected > 0)
    }

    pub fn update_wallet(&self, wallet: &Wallet) -> Result<bool> {
        // Get the wallet ID first
        let wallet_id = match self.get_wallet_by_address(&wallet.address)? {
            Some(existing_wallet) => existing_wallet.id.unwrap(),
            None => return Ok(false), // Wallet not found
        };

        // Update basic wallet fields
        let rows_affected = self.conn.execute(
            "UPDATE wallets SET
                label = ?1,
                notes = ?2
            WHERE address = ?3",
            params![
                wallet.label,
                wallet.notes,
                wallet.address
            ],
        ).context("Failed to update wallet")?;

        // Update additional data in separate table
        // First clear existing data
        self.conn.execute(
            "DELETE FROM wallet_additional_data WHERE wallet_id = ?1",
            params![wallet_id],
        ).context("Failed to clear existing additional data")?;

        // Insert new additional data
        for (key, value) in &wallet.additional_data {
            self.conn.execute(
                "INSERT INTO wallet_additional_data (wallet_id, data_key, data_value) VALUES (?1, ?2, ?3)",
                params![wallet_id, key, value],
            ).context("Failed to insert additional data")?;
        }

        // Update secondary addresses in separate table
        // First clear existing secondary addresses
        self.conn.execute(
            "DELETE FROM wallet_secondary_addresses WHERE wallet_id = ?1",
            params![wallet_id],
        ).context("Failed to clear existing secondary addresses")?;

        // Insert new secondary addresses
        for (addr_type, address) in &wallet.secondary_addresses {
            self.conn.execute(
                "INSERT INTO wallet_secondary_addresses (wallet_id, address_type, address) VALUES (?1, ?2, ?3)",
                params![wallet_id, addr_type, address],
            ).context("Failed to insert secondary address")?;
        }

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
    pub fn rename_address_group(&self, base_wallet_id: i64, old_name: &str, new_name: &str) -> Result<bool> {
        // Check if new name already exists for this base wallet
        if self.get_address_group_by_name_for_wallet(base_wallet_id, new_name)?.is_some() {
            bail!("Address group '{}' already exists for this wallet.", new_name);
        }

        let rows_affected = self.conn.execute(
            "UPDATE address_groups SET name = ?1, updated_at = CURRENT_TIMESTAMP WHERE base_wallet_id = ?2 AND name = ?3",
            params![new_name, base_wallet_id, old_name],
        ).context("Failed to rename address group")?;

        Ok(rows_affected > 0)
    }

    /// Deletes wallet group and all associated data (requires mnemonic verification)
    pub fn delete_wallet_group(&self, master_account_id: i64, group_name: &str) -> Result<bool> {
        // CLI layer has already validated mnemonic - just perform deletion
        let rows_affected = self.conn.execute(
            "DELETE FROM wallet_groups WHERE master_account_id = ?1 AND name = ?2",
            params![master_account_id, group_name],
        ).context("Failed to delete wallet group")?;

        Ok(rows_affected > 0)
    }

    /// Deletes address group and all associated addresses (requires mnemonic verification)
    pub fn delete_address_group(&self, base_wallet_id: i64, address_group_name: &str, mnemonic_verification: &str) -> Result<bool> {
        // Get master account for mnemonic verification through wallet chain
        let master_account_mnemonic: String = self.conn.query_row(
            "SELECT ma.mnemonic FROM master_accounts ma
             JOIN wallet_groups wg ON ma.id = wg.master_account_id
             JOIN wallets w ON wg.id = w.wallet_group_id
             WHERE w.id = ?1",
            [base_wallet_id],
            |row| Ok(row.get(0)?)
        ).context("Failed to get master account mnemonic")?;

        if master_account_mnemonic != mnemonic_verification {
            bail!("Mnemonic verification failed. Cannot delete address group.");
        }

        let rows_affected = self.conn.execute(
            "DELETE FROM address_groups WHERE base_wallet_id = ?1 AND name = ?2",
            params![base_wallet_id, address_group_name],
        ).context("Failed to delete address group")?;

        Ok(rows_affected > 0)
    }

    /// Deletes individual wallet (requires mnemonic verification for hierarchical wallets)
    pub fn delete_wallet(&self, address: &str, mnemonic_verification: Option<&str>) -> Result<bool> {
        // Check if this is a standalone wallet or hierarchical wallet
        let wallet_info: (Option<i64>, Option<i64>, String) = self.conn.query_row(
            "SELECT wallet_group_id, address_group_id, source_type FROM wallets WHERE address = ?1",
            [address],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        ).context("Failed to get wallet info")?;

        let (wallet_group_id, _address_group_id, source_type) = wallet_info;

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
            "DELETE FROM wallets WHERE address = ?1",
            params![address],
        ).context("Failed to delete wallet")?;

        Ok(rows_affected > 0)
    }
    /// Search wallets by term, optionally filtered by blockchain
    pub fn search_wallets(&self, term: &str, blockchain: Option<&str>) -> Result<Vec<Wallet>> {
        let (query, params): (String, Vec<String>) = match blockchain {
            Some(chain) => (
                r#"
                SELECT id, wallet_group_id, address_group_id, blockchain, address, address_with_checksum,
                       private_key, public_key, derivation_path, label,
                       source_type, explorer_url, notes, created_at
                FROM wallets
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
                       private_key, public_key, derivation_path, label,
                       source_type, explorer_url, notes, created_at
                FROM wallets
                WHERE label LIKE ?1 OR address LIKE ?1 OR blockchain LIKE ?1 OR notes LIKE ?1
                ORDER BY created_at DESC
                "#.to_string(),
                vec![format!("%{}%", term)]
            )
        };

        let mut stmt = self.conn.prepare(&query)
            .context("Failed to prepare search statement")?;

        let wallet_iter = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
            self.build_wallet_from_row(row)
        }).context("Failed to search wallets")?;

        let mut wallets = Vec::new();
        for wallet_result in wallet_iter {
            let wallet = wallet_result.context("Failed to parse wallet")?;
            let completed_wallet = self.complete_wallet(wallet)?;
            wallets.push(completed_wallet);
        }

        Ok(wallets)
    }

}