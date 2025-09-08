use rusqlite::{Connection, params, Result as SqlResult};
use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletRecord {
    pub id: Option<i64>,
    pub label: Option<String>,
    pub blockchain: String,
    pub address: String,
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
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS wallets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                label TEXT,
                blockchain TEXT NOT NULL,
                address TEXT NOT NULL UNIQUE,
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
        
        Ok(())
    }
    
    pub fn insert_wallet(&self, wallet: &WalletRecord) -> Result<i64> {
        let mut stmt = self.conn.prepare(
            r#"
            INSERT INTO wallets (
                label, blockchain, address, public_key, private_key,
                mnemonic, passphrase, derivation_path, account, address_index,
                source_type, explorer_url, imported_at, notes
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            "#
        ).context("Failed to prepare insert statement")?;
        
        stmt.execute(params![
            wallet.label,
            wallet.blockchain,
            wallet.address,
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
        
        Ok(self.conn.last_insert_rowid())
    }
    
    pub fn get_all_wallets(&self) -> Result<Vec<WalletRecord>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, label, blockchain, address, public_key, private_key,
                   mnemonic, passphrase, derivation_path, account, address_index,
                   source_type, explorer_url, imported_at, notes
            FROM wallets ORDER BY imported_at DESC
            "#
        ).context("Failed to prepare select statement")?;
        
        let wallet_iter = stmt.query_map([], |row| {
            Ok(WalletRecord {
                id: Some(row.get(0)?),
                label: row.get(1)?,
                blockchain: row.get(2)?,
                address: row.get(3)?,
                public_key: row.get(4)?,
                private_key: row.get(5)?,
                mnemonic: row.get(6)?,
                passphrase: row.get(7)?,
                derivation_path: row.get(8)?,
                account: row.get(9)?,
                address_index: row.get(10)?,
                source_type: row.get(11)?,
                explorer_url: row.get(12)?,
                imported_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(13)?)
                    .unwrap().with_timezone(&Utc),
                notes: row.get(14)?,
            })
        }).context("Failed to query wallets")?;
        
        let mut wallets = Vec::new();
        for wallet in wallet_iter {
            wallets.push(wallet.context("Failed to parse wallet record")?);
        }
        
        Ok(wallets)
    }
    
    pub fn get_wallet_by_address(&self, address: &str) -> Result<Option<WalletRecord>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, label, blockchain, address, public_key, private_key,
                   mnemonic, passphrase, derivation_path, account, address_index,
                   source_type, explorer_url, imported_at, notes
            FROM wallets WHERE address = ?1
            "#
        ).context("Failed to prepare select statement")?;
        
        let mut rows = stmt.query_map([address], |row| {
            Ok(WalletRecord {
                id: Some(row.get(0)?),
                label: row.get(1)?,
                blockchain: row.get(2)?,
                address: row.get(3)?,
                public_key: row.get(4)?,
                private_key: row.get(5)?,
                mnemonic: row.get(6)?,
                passphrase: row.get(7)?,
                derivation_path: row.get(8)?,
                account: row.get(9)?,
                address_index: row.get(10)?,
                source_type: row.get(11)?,
                explorer_url: row.get(12)?,
                imported_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(13)?)
                    .unwrap().with_timezone(&Utc),
                notes: row.get(14)?,
            })
        }).context("Failed to query wallet")?;
        
        match rows.next() {
            Some(wallet) => Ok(Some(wallet.context("Failed to parse wallet record")?)),
            None => Ok(None),
        }
    }
    
    pub fn get_wallet_by_label(&self, label: &str) -> Result<Option<WalletRecord>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, label, blockchain, address, public_key, private_key,
                   mnemonic, passphrase, derivation_path, account, address_index,
                   source_type, explorer_url, imported_at, notes
            FROM wallets WHERE label = ?1
            "#
        ).context("Failed to prepare select statement")?;
        
        let mut rows = stmt.query_map([label], |row| {
            Ok(WalletRecord {
                id: Some(row.get(0)?),
                label: row.get(1)?,
                blockchain: row.get(2)?,
                address: row.get(3)?,
                public_key: row.get(4)?,
                private_key: row.get(5)?,
                mnemonic: row.get(6)?,
                passphrase: row.get(7)?,
                derivation_path: row.get(8)?,
                account: row.get(9)?,
                address_index: row.get(10)?,
                source_type: row.get(11)?,
                explorer_url: row.get(12)?,
                imported_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(13)?)
                    .unwrap().with_timezone(&Utc),
                notes: row.get(14)?,
            })
        }).context("Failed to query wallet")?;
        
        match rows.next() {
            Some(wallet) => Ok(Some(wallet.context("Failed to parse wallet record")?)),
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
                SELECT id, label, blockchain, address, public_key, private_key,
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
                SELECT id, label, blockchain, address, public_key, private_key,
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
        
        let wallet_iter = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
            Ok(WalletRecord {
                id: Some(row.get(0)?),
                label: row.get(1)?,
                blockchain: row.get(2)?,
                address: row.get(3)?,
                public_key: row.get(4)?,
                private_key: row.get(5)?,
                mnemonic: row.get(6)?,
                passphrase: row.get(7)?,
                derivation_path: row.get(8)?,
                account: row.get(9)?,
                address_index: row.get(10)?,
                source_type: row.get(11)?,
                explorer_url: row.get(12)?,
                imported_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(13)?)
                    .unwrap().with_timezone(&Utc),
                notes: row.get(14)?,
            })
        }).context("Failed to search wallets")?;
        
        let mut wallets = Vec::new();
        for wallet in wallet_iter {
            wallets.push(wallet.context("Failed to parse wallet record")?);
        }
        
        Ok(wallets)
    }
}