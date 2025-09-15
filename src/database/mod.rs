use rusqlite::{Connection, params, Result as SqlResult};
use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
        // Enhanced wallets table with checksum support
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
                notes TEXT
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
                    source_type, explorer_url, imported_at, notes
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
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
                   source_type, explorer_url, imported_at, notes
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
                   source_type, explorer_url, imported_at, notes
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
                       source_type, explorer_url, imported_at, notes
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
                       source_type, explorer_url, imported_at, notes
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
}