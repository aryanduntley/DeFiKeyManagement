use rusqlite::{Connection, params, Result as SqlResult};
use anyhow::{Result, Context, bail};
use chrono::{DateTime, Utc, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletRecord {
    pub id: Option<i64>,
    pub label: Option<String>,
    pub blockchain: String,
    pub address: String,
    pub address_with_checksum: Option<String>, // New: checksummed addresses
    pub public_key: Option<String>,
    pub private_key: String,
    pub mnemonic: Option<String>,
    pub passphrase: Option<String>,
    pub derivation_path: String,
    pub account: Option<u32>,
    pub address_index: Option<u32>,
    pub source_type: String, // "mnemonic" or "private_key"
    pub explorer_url: Option<String>,
    pub imported_at: DateTime<Utc>,
    pub notes: Option<String>,
    pub additional_data: HashMap<String, String>, // New: blockchain-specific data
    pub secondary_addresses: HashMap<String, String>, // New: secondary addresses
    pub group_id: Option<i64>, // New: link to wallet group
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletGroup {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub mnemonic_hash: String, // SHA-256 hash of mnemonic for privacy
    pub blockchains: Vec<String>, // List of blockchains user selected for this group
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletGroupSummary {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub blockchains: Vec<String>,
    pub wallet_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
        // Create wallet_groups table first (referenced by wallets)
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS wallet_groups (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                description TEXT,
                mnemonic_hash TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            "#,
            [],
        ).context("Failed to create wallet_groups table")?;

        // Create wallet_group_blockchains table to store which blockchains each group supports
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS wallet_group_blockchains (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                group_id INTEGER NOT NULL,
                blockchain TEXT NOT NULL,
                FOREIGN KEY (group_id) REFERENCES wallet_groups(id) ON DELETE CASCADE,
                UNIQUE(group_id, blockchain)
            );
            "#,
            [],
        ).context("Failed to create wallet_group_blockchains table")?;

        // Enhanced wallets table with checksum support and group_id
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS wallets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                label TEXT,
                blockchain TEXT NOT NULL,
                address TEXT NOT NULL UNIQUE,
                address_with_checksum TEXT,
                public_key TEXT,
                private_key TEXT NOT NULL,
                mnemonic TEXT,
                passphrase TEXT,
                derivation_path TEXT NOT NULL,
                account INTEGER,
                address_index INTEGER,
                source_type TEXT NOT NULL,
                explorer_url TEXT,
                imported_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                notes TEXT,
                group_id INTEGER,
                FOREIGN KEY (group_id) REFERENCES wallet_groups(id) ON DELETE SET NULL
            );
            "#,
            [],
        ).context("Failed to create wallets table")?;

        // Table for blockchain-specific additional data
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

        // Table for secondary addresses (EVM, legacy, etc.)
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

        // Create indexes for performance
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_wallets_blockchain ON wallets(blockchain);",
            [],
        ).context("Failed to create blockchain index")?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_wallets_label ON wallets(label);",
            [],
        ).context("Failed to create label index")?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_wallets_address ON wallets(address);",
            [],
        ).context("Failed to create address index")?;

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

        // Create wallet groups indexes
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_wallet_groups_name ON wallet_groups(name);",
            [],
        ).context("Failed to create wallet groups name index")?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_wallet_groups_mnemonic_hash ON wallet_groups(mnemonic_hash);",
            [],
        ).context("Failed to create wallet groups mnemonic hash index")?;

        // Create wallet group_id index
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_wallets_group_id ON wallets(group_id);",
            [],
        ).context("Failed to create wallets group_id index")?;

        // Create wallet group blockchains indexes
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_wallet_group_blockchains_group_id ON wallet_group_blockchains(group_id);",
            [],
        ).context("Failed to create wallet group blockchains group_id index")?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_wallet_group_blockchains_blockchain ON wallet_group_blockchains(blockchain);",
            [],
        ).context("Failed to create wallet group blockchains blockchain index")?;

        Ok(())
    }
    
    pub fn insert_wallet(&self, wallet: &WalletRecord) -> Result<i64> {
        // Start transaction for inserting wallet and related data
        let tx = self.conn.unchecked_transaction()?;

        // Insert main wallet record
        {
            let mut stmt = tx.prepare(
                r#"
                INSERT INTO wallets (
                    label, blockchain, address, address_with_checksum, public_key, private_key,
                    mnemonic, passphrase, derivation_path, account, address_index,
                    source_type, explorer_url, imported_at, notes, group_id
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
                "#
            ).context("Failed to prepare insert statement")?;

            stmt.execute(params![
                wallet.label,
                wallet.blockchain,
                wallet.address,
                wallet.address_with_checksum,
                wallet.public_key,
                wallet.private_key,
                wallet.mnemonic,
                wallet.passphrase,
                wallet.derivation_path,
                wallet.account,
                wallet.address_index,
                wallet.source_type,
                wallet.explorer_url,
                wallet.imported_at.to_rfc3339(),
                wallet.notes,
                wallet.group_id,
            ]).context("Failed to execute insert")?;
        }

        let wallet_id = tx.last_insert_rowid();

        // Insert additional data
        {
            let mut data_stmt = tx.prepare(
                "INSERT INTO wallet_additional_data (wallet_id, data_key, data_value) VALUES (?1, ?2, ?3)"
            ).context("Failed to prepare additional data insert")?;

            for (key, value) in &wallet.additional_data {
                data_stmt.execute(params![wallet_id, key, value])
                    .context("Failed to insert additional data")?;
            }
        }

        // Insert secondary addresses
        {
            let mut addr_stmt = tx.prepare(
                "INSERT INTO wallet_secondary_addresses (wallet_id, address_type, address) VALUES (?1, ?2, ?3)"
            ).context("Failed to prepare secondary address insert")?;

            for (addr_type, address) in &wallet.secondary_addresses {
                addr_stmt.execute(params![wallet_id, addr_type, address])
                    .context("Failed to insert secondary address")?;
            }
        }

        tx.commit().context("Failed to commit transaction")?;
        Ok(wallet_id)
    }

    // Helper method to load additional data for a wallet
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

    // Helper method to load secondary addresses for a wallet
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

    // Helper method to construct WalletRecord from database row
    fn build_wallet_record(&self, row: &rusqlite::Row) -> SqlResult<WalletRecord> {
        let wallet_id: i64 = row.get(0)?;
        Ok(WalletRecord {
            id: Some(wallet_id),
            label: row.get(1)?,
            blockchain: row.get(2)?,
            address: row.get(3)?,
            address_with_checksum: row.get(4)?,
            public_key: row.get(5)?,
            private_key: row.get(6)?,
            mnemonic: row.get(7)?,
            passphrase: row.get(8)?,
            derivation_path: row.get(9)?,
            account: row.get(10)?,
            address_index: row.get(11)?,
            source_type: row.get(12)?,
            explorer_url: row.get(13)?,
            imported_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(14)?)
                .unwrap().with_timezone(&Utc),
            notes: row.get(15)?,
            group_id: row.get(16)?,
            additional_data: HashMap::new(),
            secondary_addresses: HashMap::new(),
        })
    }

    // Helper method to complete wallet record with additional data
    fn complete_wallet_record(&self, mut wallet: WalletRecord) -> Result<WalletRecord> {
        if let Some(wallet_id) = wallet.id {
            wallet.additional_data = self.load_additional_data(wallet_id)?;
            wallet.secondary_addresses = self.load_secondary_addresses(wallet_id)?;
        }
        Ok(wallet)
    }
    
    pub fn get_all_wallets(&self) -> Result<Vec<WalletRecord>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, label, blockchain, address, address_with_checksum, public_key, private_key,
                   mnemonic, passphrase, derivation_path, account, address_index,
                   source_type, explorer_url, imported_at, notes, group_id
            FROM wallets ORDER BY imported_at DESC
            "#
        ).context("Failed to prepare select statement")?;

        let wallet_iter = stmt.query_map([], |row| self.build_wallet_record(row))
            .context("Failed to query wallets")?;

        let mut wallets = Vec::new();
        for wallet_result in wallet_iter {
            let wallet = wallet_result.context("Failed to parse wallet record")?;
            let completed_wallet = self.complete_wallet_record(wallet)?;
            wallets.push(completed_wallet);
        }

        Ok(wallets)
    }
    
    pub fn get_wallet_by_address(&self, address: &str) -> Result<Option<WalletRecord>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, label, blockchain, address, address_with_checksum, public_key, private_key,
                   mnemonic, passphrase, derivation_path, account, address_index,
                   source_type, explorer_url, imported_at, notes, group_id
            FROM wallets WHERE address = ?1
            "#
        ).context("Failed to prepare select statement")?;

        let mut rows = stmt.query_map([address], |row| self.build_wallet_record(row))
            .context("Failed to query wallet")?;

        match rows.next() {
            Some(wallet) => {
                let wallet = wallet.context("Failed to parse wallet record")?;
                let completed_wallet = self.complete_wallet_record(wallet)?;
                Ok(Some(completed_wallet))
            },
            None => Ok(None),
        }
    }
    
    pub fn get_wallet_by_label(&self, label: &str) -> Result<Option<WalletRecord>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, label, blockchain, address, address_with_checksum, public_key, private_key,
                   mnemonic, passphrase, derivation_path, account, address_index,
                   source_type, explorer_url, imported_at, notes
            FROM wallets WHERE label = ?1
            "#
        ).context("Failed to prepare select statement")?;

        let mut rows = stmt.query_map([label], |row| self.build_wallet_record(row))
            .context("Failed to query wallet")?;

        match rows.next() {
            Some(wallet) => {
                let wallet = wallet.context("Failed to parse wallet record")?;
                let completed_wallet = self.complete_wallet_record(wallet)?;
                Ok(Some(completed_wallet))
            },
            None => Ok(None),
        }
    }
    
    pub fn update_wallet_label(&self, address: &str, new_label: &str) -> Result<bool> {
        let rows_affected = self.conn.execute(
            "UPDATE wallets SET label = ?1 WHERE address = ?2",
            params![new_label, address],
        ).context("Failed to update wallet label")?;
        
        Ok(rows_affected > 0)
    }
    
    pub fn delete_wallet_by_address(&self, address: &str) -> Result<bool> {
        let rows_affected = self.conn.execute(
            "DELETE FROM wallets WHERE address = ?1",
            params![address],
        ).context("Failed to delete wallet")?;
        
        Ok(rows_affected > 0)
    }
    
    pub fn delete_wallet_by_label(&self, label: &str) -> Result<bool> {
        let rows_affected = self.conn.execute(
            "DELETE FROM wallets WHERE label = ?1",
            params![label],
        ).context("Failed to delete wallet")?;
        
        Ok(rows_affected > 0)
    }
    
    pub fn search_wallets(&self, term: &str, blockchain: Option<&str>) -> Result<Vec<WalletRecord>> {
        let (query, params): (String, Vec<String>) = match blockchain {
            Some(chain) => (
                r#"
                SELECT id, label, blockchain, address, address_with_checksum, public_key, private_key,
                       mnemonic, passphrase, derivation_path, account, address_index,
                       source_type, explorer_url, imported_at, notes, group_id
                FROM wallets
                WHERE blockchain = ?1 AND (
                    label LIKE ?2 OR
                    address LIKE ?2 OR
                    blockchain LIKE ?2
                )
                ORDER BY imported_at DESC
                "#.to_string(),
                vec![chain.to_string(), format!("%{}%", term)]
            ),
            None => (
                r#"
                SELECT id, label, blockchain, address, address_with_checksum, public_key, private_key,
                       mnemonic, passphrase, derivation_path, account, address_index,
                       source_type, explorer_url, imported_at, notes, group_id
                FROM wallets
                WHERE label LIKE ?1 OR address LIKE ?1 OR blockchain LIKE ?1
                ORDER BY imported_at DESC
                "#.to_string(),
                vec![format!("%{}%", term)]
            )
        };

        let mut stmt = self.conn.prepare(&query)
            .context("Failed to prepare search statement")?;

        let wallet_iter = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| self.build_wallet_record(row))
            .context("Failed to search wallets")?;

        let mut wallets = Vec::new();
        for wallet_result in wallet_iter {
            let wallet = wallet_result.context("Failed to parse wallet record")?;
            let completed_wallet = self.complete_wallet_record(wallet)?;
            wallets.push(completed_wallet);
        }

        Ok(wallets)
    }

    // ========== WALLET GROUP MANAGEMENT ==========

    /// Generates a SHA-256 hash of the mnemonic for privacy
    fn hash_mnemonic(mnemonic: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(mnemonic.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Creates or finds existing wallet group for the given mnemonic and blockchains
    pub fn create_or_get_wallet_group(&self, name: &str, description: Option<&str>, mnemonic: &str, blockchains: &[String]) -> Result<i64> {
        let mnemonic_hash = Self::hash_mnemonic(mnemonic);

        // Check if group with this name already exists
        if let Some(existing_group) = self.get_wallet_group_by_name(name)? {
            // Verify the mnemonic matches (security check)
            if existing_group.mnemonic_hash != mnemonic_hash {
                bail!("Group '{}' already exists with a different mnemonic. Please choose a different group name.", name);
            }
            return Ok(existing_group.id.unwrap());
        }

        // Start transaction for creating group and blockchain associations
        let tx = self.conn.unchecked_transaction()?;

        // Insert wallet group
        let group_id = {
            let mut stmt = tx.prepare(
                "INSERT INTO wallet_groups (name, description, mnemonic_hash) VALUES (?1, ?2, ?3)"
            ).context("Failed to prepare wallet group insert")?;

            stmt.execute(params![name, description, mnemonic_hash])
                .context("Failed to insert wallet group")?;

            tx.last_insert_rowid()
        };

        // Insert blockchain associations
        {
            let mut stmt = tx.prepare(
                "INSERT INTO wallet_group_blockchains (group_id, blockchain) VALUES (?1, ?2)"
            ).context("Failed to prepare blockchain insert")?;

            for blockchain in blockchains {
                stmt.execute(params![group_id, blockchain])
                    .context("Failed to insert blockchain association")?;
            }
        }

        tx.commit().context("Failed to commit wallet group transaction")?;
        Ok(group_id)
    }

    /// Gets wallet group by name
    pub fn get_wallet_group_by_name(&self, name: &str) -> Result<Option<WalletGroup>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, mnemonic_hash, created_at, updated_at FROM wallet_groups WHERE name = ?1"
        ).context("Failed to prepare wallet group query")?;

        let group_result = stmt.query_row([name], |row| {
            Ok(WalletGroup {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                mnemonic_hash: row.get(3)?,
                blockchains: Vec::new(), // Will be loaded separately
                created_at: {
                    let datetime_str: String = row.get(4)?;
                    if datetime_str.contains('T') {
                        DateTime::parse_from_rfc3339(&datetime_str).unwrap().with_timezone(&Utc)
                    } else {
                        NaiveDateTime::parse_from_str(&datetime_str, "%Y-%m-%d %H:%M:%S")
                            .unwrap().and_utc()
                    }
                },
                updated_at: {
                    let datetime_str: String = row.get(5)?;
                    if datetime_str.contains('T') {
                        DateTime::parse_from_rfc3339(&datetime_str).unwrap().with_timezone(&Utc)
                    } else {
                        NaiveDateTime::parse_from_str(&datetime_str, "%Y-%m-%d %H:%M:%S")
                            .unwrap().and_utc()
                    }
                },
            })
        });

        match group_result {
            Ok(mut group) => {
                // Load associated blockchains
                if let Some(group_id) = group.id {
                    group.blockchains = self.get_group_blockchains(group_id)?;
                }
                Ok(Some(group))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(anyhow::Error::from(e).context("Failed to query wallet group")),
        }
    }

    /// Gets all wallet groups with summary information
    pub fn get_all_wallet_groups(&self) -> Result<Vec<WalletGroupSummary>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                g.id, g.name, g.description, g.created_at, g.updated_at,
                COUNT(w.id) as wallet_count
            FROM wallet_groups g
            LEFT JOIN wallets w ON g.id = w.group_id
            GROUP BY g.id, g.name, g.description, g.created_at, g.updated_at
            ORDER BY g.created_at DESC
            "#
        ).context("Failed to prepare wallet groups query")?;

        let group_iter = stmt.query_map([], |row| {
            Ok(WalletGroupSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                blockchains: Vec::new(), // Will be loaded separately
                wallet_count: row.get(5)?,
                created_at: {
                    let datetime_str: String = row.get(3)?;
                    if datetime_str.contains('T') {
                        // RFC3339 format
                        DateTime::parse_from_rfc3339(&datetime_str).unwrap().with_timezone(&Utc)
                    } else {
                        // SQLite CURRENT_TIMESTAMP format: "YYYY-MM-DD HH:MM:SS"
                        chrono::NaiveDateTime::parse_from_str(&datetime_str, "%Y-%m-%d %H:%M:%S")
                            .unwrap().and_utc()
                    }
                },
                updated_at: {
                    let datetime_str: String = row.get(4)?;
                    if datetime_str.contains('T') {
                        // RFC3339 format
                        DateTime::parse_from_rfc3339(&datetime_str).unwrap().with_timezone(&Utc)
                    } else {
                        // SQLite CURRENT_TIMESTAMP format: "YYYY-MM-DD HH:MM:SS"
                        chrono::NaiveDateTime::parse_from_str(&datetime_str, "%Y-%m-%d %H:%M:%S")
                            .unwrap().and_utc()
                    }
                },
            })
        }).context("Failed to query wallet groups")?;

        let mut groups = Vec::new();
        for group_result in group_iter {
            let mut group = group_result.context("Failed to parse wallet group")?;
            group.blockchains = self.get_group_blockchains(group.id)?;
            groups.push(group);
        }

        Ok(groups)
    }

    /// Gets wallets for a specific group
    pub fn get_wallets_by_group_id(&self, group_id: i64) -> Result<Vec<WalletRecord>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, label, blockchain, address, address_with_checksum, public_key, private_key,
                   mnemonic, passphrase, derivation_path, account, address_index,
                   source_type, explorer_url, imported_at, notes, group_id
            FROM wallets
            WHERE group_id = ?1
            ORDER BY blockchain, address_index
            "#
        ).context("Failed to prepare wallet group query")?;

        let wallet_iter = stmt.query_map([group_id], |row| self.build_wallet_record(row))
            .context("Failed to query wallets by group")?;

        let mut wallets = Vec::new();
        for wallet_result in wallet_iter {
            let wallet = wallet_result.context("Failed to parse wallet record")?;
            let completed_wallet = self.complete_wallet_record(wallet)?;
            wallets.push(completed_wallet);
        }

        Ok(wallets)
    }

    /// Helper method to get blockchains associated with a group
    fn get_group_blockchains(&self, group_id: i64) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT blockchain FROM wallet_group_blockchains WHERE group_id = ?1 ORDER BY blockchain"
        ).context("Failed to prepare group blockchains query")?;

        let blockchain_iter = stmt.query_map([group_id], |row| Ok(row.get::<_, String>(0)?))
            .context("Failed to query group blockchains")?;

        let mut blockchains = Vec::new();
        for blockchain_result in blockchain_iter {
            blockchains.push(blockchain_result.context("Failed to parse blockchain")?);
        }

        Ok(blockchains)
    }

    /// Updates wallet group blockchain associations (for extending existing groups)
    pub fn add_blockchains_to_group(&self, group_id: i64, new_blockchains: &[String]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;

        {
            let mut stmt = tx.prepare(
                "INSERT OR IGNORE INTO wallet_group_blockchains (group_id, blockchain) VALUES (?1, ?2)"
            ).context("Failed to prepare blockchain insert")?;

            for blockchain in new_blockchains {
                stmt.execute(params![group_id, blockchain])
                    .context("Failed to insert blockchain association")?;
            }
        }

        // Update the group's updated_at timestamp
        tx.execute(
            "UPDATE wallet_groups SET updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
            params![group_id],
        ).context("Failed to update group timestamp")?;

        tx.commit().context("Failed to commit blockchain additions")?;
        Ok(())
    }

    /// Deletes a wallet group and all associated wallets
    pub fn delete_wallet_group(&self, group_id: i64) -> Result<bool> {
        let tx = self.conn.unchecked_transaction()?;

        // Delete all wallets in the group (CASCADE will handle additional_data and secondary_addresses)
        tx.execute(
            "DELETE FROM wallets WHERE group_id = ?1",
            params![group_id],
        ).context("Failed to delete group wallets")?;

        // Delete the group (CASCADE will handle blockchain associations)
        let rows_affected = tx.execute(
            "DELETE FROM wallet_groups WHERE id = ?1",
            params![group_id],
        ).context("Failed to delete wallet group")?;

        tx.commit().context("Failed to commit group deletion")?;
        Ok(rows_affected > 0)
    }

    /// Renames a wallet group
    pub fn rename_wallet_group(&self, old_name: &str, new_name: &str) -> Result<bool> {
        // Check if new name already exists
        if self.get_wallet_group_by_name(new_name)?.is_some() {
            bail!("Group '{}' already exists. Please choose a different name.", new_name);
        }

        let rows_affected = self.conn.execute(
            "UPDATE wallet_groups SET name = ?1, updated_at = CURRENT_TIMESTAMP WHERE name = ?2",
            params![new_name, old_name],
        ).context("Failed to rename wallet group")?;

        Ok(rows_affected > 0)
    }
}