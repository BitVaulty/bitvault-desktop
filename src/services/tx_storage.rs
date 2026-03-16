//! TxStorage - SQLite transaction storage
//!
//! Equivalent to Swift's SQLiteTransactionStorage
//! Stores transaction history, sync records, and recent addresses

use bitvault_common::types::Price;
use dirs;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Transaction status constants
pub mod tx_status {
    pub const CONFIRMED: &str = "confirmed";
    pub const PENDING: &str = "pending";
}

/// Transaction details
/// Equivalent to Swift's TxDetails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxDetails {
    pub tx_id: String,
    pub amount_sent: f64,
    pub amount_received: f64,
    pub fee_in_sat: Option<i64>,
    pub prices: Price,
    pub block_height: Option<i64>,
    pub timestamp: Option<i64>,
    pub status: String, // "confirmed" or "pending"
    pub destination_address: Option<String>,
    pub source_address: Option<String>,
    pub address: String,
    pub locktime: i64,
    pub message: Option<String>,
    pub transaction_execution_timestamp: i64,
}

/// Sync record for tracking last synced transaction
/// Equivalent to Swift's SyncRecord
#[derive(Debug, Clone)]
pub struct SyncRecord {
    pub tx_id: String,
    pub timestamp: i64,
}

/// Recent address entry
#[derive(Debug, Clone)]
pub struct RecentAddress {
    pub recipient_address: String,
    pub timestamp: i64,
}

/// SQLite transaction storage
/// Equivalent to Swift's SQLiteTransactionStorage
pub struct SQLiteTransactionStorage {
    db: Connection,
}

impl SQLiteTransactionStorage {
    /// Create a new transaction storage instance
    /// Creates database file in application support directory
    pub fn new() -> Result<Self, SQLiteTransactionStorageError> {
        let db_path = Self::get_db_path()?;

        let db = Connection::open(&db_path)
            .map_err(|e| SQLiteTransactionStorageError::ConnectionFailed(e.to_string()))?;

        let storage = Self { db };
        storage.create_tables()?;

        Ok(storage)
    }

    /// Get database path
    fn get_db_path() -> Result<PathBuf, SQLiteTransactionStorageError> {
        let app_support_dir = dirs::data_dir().ok_or_else(|| {
            SQLiteTransactionStorageError::PathError("Failed to get data directory".to_string())
        })?;

        // Create bitvault directory if it doesn't exist
        let bitvault_dir = app_support_dir.join("bitvault");
        std::fs::create_dir_all(&bitvault_dir)
            .map_err(|e| SQLiteTransactionStorageError::PathError(e.to_string()))?;

        Ok(bitvault_dir.join("transactions.sqlite"))
    }

    /// Create database tables
    fn create_tables(&self) -> Result<(), SQLiteTransactionStorageError> {
        // Create transactions table
        self.db
            .execute(
                "CREATE TABLE IF NOT EXISTS transactions (
                txId TEXT NOT NULL,
                amountSent REAL NOT NULL,
                amountReceived REAL NOT NULL,
                fee INTEGER,
                prices TEXT,
                blockHeight INTEGER,
                timestamp INTEGER,
                status TEXT NOT NULL,
                destinationAddress TEXT,
                sourceAddress TEXT,
                address TEXT NOT NULL,
                locktime INTEGER NOT NULL,
                message TEXT,
                transactionExecutionTimestamp INTEGER NOT NULL,
                PRIMARY KEY (txId, address)
            )",
                [],
            )
            .map_err(|e| {
                SQLiteTransactionStorageError::SchemaCreationFailed(
                    "transactions".to_string(),
                    e.to_string(),
                )
            })?;

        // Create sync record table
        self.db
            .execute(
                "CREATE TABLE IF NOT EXISTS syncRecord (
                syncAddress TEXT PRIMARY KEY,
                syncTxId TEXT NOT NULL,
                syncTimestamp INTEGER NOT NULL
            )",
                [],
            )
            .map_err(|e| {
                SQLiteTransactionStorageError::SchemaCreationFailed(
                    "syncRecord".to_string(),
                    e.to_string(),
                )
            })?;

        // Create recent addresses table
        self.db
            .execute(
                "CREATE TABLE IF NOT EXISTS recentAddresses (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                vault TEXT NOT NULL,
                recipientAddress TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )",
                [],
            )
            .map_err(|e| {
                SQLiteTransactionStorageError::SchemaCreationFailed(
                    "recentAddresses".to_string(),
                    e.to_string(),
                )
            })?;

        Ok(())
    }

    /// Insert a transaction
    /// Equivalent to Swift's insert
    pub fn insert(&self, tx: &TxDetails) -> Result<(), SQLiteTransactionStorageError> {
        let prices_json = serde_json::to_string(&tx.prices)
            .map_err(|e| SQLiteTransactionStorageError::SerializationError(e.to_string()))?;

        self.db
            .execute(
                "INSERT OR REPLACE INTO transactions (
                txId, amountSent, amountReceived, fee, prices, blockHeight, timestamp,
                status, destinationAddress, sourceAddress, address, locktime, message,
                transactionExecutionTimestamp
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                params![
                    tx.tx_id,
                    tx.amount_sent,
                    tx.amount_received,
                    tx.fee_in_sat,
                    prices_json,
                    tx.block_height,
                    tx.timestamp,
                    tx.status,
                    tx.destination_address,
                    tx.source_address,
                    tx.address,
                    tx.locktime,
                    tx.message,
                    tx.transaction_execution_timestamp,
                ],
            )
            .map_err(|e| SQLiteTransactionStorageError::InsertError(e.to_string()))?;

        Ok(())
    }

    /// Get all transactions for an address
    /// Equivalent to Swift's getAllTxs
    pub fn get_all_txs(
        &self,
        address: &str,
    ) -> Result<Vec<TxDetails>, SQLiteTransactionStorageError> {
        let mut stmt = self
            .db
            .prepare(
                "SELECT txId, amountSent, amountReceived, fee, prices, blockHeight, timestamp,
                    status, destinationAddress, sourceAddress, address, locktime, message,
                    transactionExecutionTimestamp
             FROM transactions
             WHERE address = ?1
             ORDER BY CASE WHEN status = 'confirmed' THEN 0 ELSE 1 END, timestamp DESC",
            )
            .map_err(|e| SQLiteTransactionStorageError::QueryError(e.to_string()))?;

        let tx_iter = stmt
            .query_map([address], |row| {
                let prices_json: String = row.get(4)?;
                let prices: Price =
                    serde_json::from_str(&prices_json).unwrap_or_else(|_| Price::default());

                Ok(TxDetails {
                    tx_id: row.get(0)?,
                    amount_sent: row.get(1)?,
                    amount_received: row.get(2)?,
                    fee_in_sat: row.get(3)?,
                    prices,
                    block_height: row.get(5)?,
                    timestamp: row.get(6)?,
                    status: row.get(7)?,
                    destination_address: row.get(8)?,
                    source_address: row.get(9)?,
                    address: row.get(10)?,
                    locktime: row.get(11)?,
                    message: row.get(12)?,
                    transaction_execution_timestamp: row.get(13)?,
                })
            })
            .map_err(|e| SQLiteTransactionStorageError::QueryError(e.to_string()))?;

        let mut results = Vec::new();
        for tx in tx_iter {
            results.push(tx.map_err(|e| SQLiteTransactionStorageError::QueryError(e.to_string()))?);
        }

        Ok(results)
    }

    /// Delete all transactions for an address
    /// Equivalent to Swift's deleteAllTxsForAddress
    pub fn delete_all_txs_for_address(
        &self,
        address: &str,
    ) -> Result<(), SQLiteTransactionStorageError> {
        self.db
            .execute("DELETE FROM transactions WHERE address = ?1", [address])
            .map_err(|e| SQLiteTransactionStorageError::DeleteError(e.to_string()))?;

        Ok(())
    }

    /// Save sync record
    /// Equivalent to Swift's saveSyncRecord
    pub fn save_sync_record(
        &self,
        tx_id: &str,
        timestamp: i64,
        address: &str,
    ) -> Result<(), SQLiteTransactionStorageError> {
        // Delete existing record first
        self.db
            .execute("DELETE FROM syncRecord WHERE syncAddress = ?1", [address])
            .map_err(|e| SQLiteTransactionStorageError::DeleteError(e.to_string()))?;

        // Insert new record
        self.db
            .execute(
                "INSERT INTO syncRecord (syncAddress, syncTxId, syncTimestamp) VALUES (?1, ?2, ?3)",
                params![address, tx_id, timestamp],
            )
            .map_err(|e| SQLiteTransactionStorageError::InsertError(e.to_string()))?;

        Ok(())
    }

    /// Get sync record
    /// Equivalent to Swift's getSyncRecord
    pub fn get_sync_record(
        &self,
        address: &str,
    ) -> Result<Option<SyncRecord>, SQLiteTransactionStorageError> {
        let mut stmt = self
            .db
            .prepare("SELECT syncTxId, syncTimestamp FROM syncRecord WHERE syncAddress = ?1")
            .map_err(|e| SQLiteTransactionStorageError::QueryError(e.to_string()))?;

        let mut rows = stmt
            .query_map([address], |row| {
                Ok(SyncRecord {
                    tx_id: row.get(0)?,
                    timestamp: row.get(1)?,
                })
            })
            .map_err(|e| SQLiteTransactionStorageError::QueryError(e.to_string()))?;

        match rows.next() {
            Some(row) => Ok(Some(row.map_err(|e| {
                SQLiteTransactionStorageError::QueryError(e.to_string())
            })?)),
            None => Ok(None),
        }
    }

    /// Delete sync record
    /// Equivalent to Swift's deleteSyncRecord
    pub fn delete_sync_record(&self, address: &str) -> Result<(), SQLiteTransactionStorageError> {
        self.db
            .execute("DELETE FROM syncRecord WHERE syncAddress = ?1", [address])
            .map_err(|e| SQLiteTransactionStorageError::DeleteError(e.to_string()))?;

        Ok(())
    }

    /// Add recent address
    /// Equivalent to Swift's addRecentAddress
    /// If the address already exists for this vault, deletes the old entry and inserts a new one with updated timestamp
    pub fn add_recent_address(
        &self,
        address: &str,
        vault: &str,
    ) -> Result<(), SQLiteTransactionStorageError> {
        // Delete existing entry first to avoid duplicates (matches Swift implementation)
        // This ensures we update the timestamp by deleting and re-inserting
        self.db
            .execute(
                "DELETE FROM recentAddresses WHERE recipientAddress = ?1 AND vault = ?2",
                params![address, vault],
            )
            .map_err(|e| SQLiteTransactionStorageError::DeleteError(e.to_string()))?;

        // Insert new entry with current timestamp
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        self.db.execute(
            "INSERT INTO recentAddresses (vault, recipientAddress, timestamp) VALUES (?1, ?2, ?3)",
            params![vault, address, timestamp],
        )
        .map_err(|e| SQLiteTransactionStorageError::InsertError(e.to_string()))?;

        Ok(())
    }

    /// Get recent addresses for a vault
    /// Equivalent to Swift's getRecentAddresses
    pub fn get_recent_addresses(
        &self,
        vault: &str,
    ) -> Result<Vec<RecentAddress>, SQLiteTransactionStorageError> {
        let mut stmt = self.db.prepare(
            "SELECT recipientAddress, timestamp FROM recentAddresses WHERE vault = ?1 ORDER BY timestamp DESC"
        )
        .map_err(|e| SQLiteTransactionStorageError::QueryError(e.to_string()))?;

        let addr_iter = stmt
            .query_map([vault], |row| {
                Ok(RecentAddress {
                    recipient_address: row.get(0)?,
                    timestamp: row.get(1)?,
                })
            })
            .map_err(|e| SQLiteTransactionStorageError::QueryError(e.to_string()))?;

        let mut results = Vec::new();
        for addr in addr_iter {
            results
                .push(addr.map_err(|e| SQLiteTransactionStorageError::QueryError(e.to_string()))?);
        }

        Ok(results)
    }

    /// Delete all addresses for a vault
    /// Equivalent to Swift's deleteAllAddresses
    pub fn delete_all_addresses(&self, vault: &str) -> Result<(), SQLiteTransactionStorageError> {
        self.db
            .execute("DELETE FROM recentAddresses WHERE vault = ?1", [vault])
            .map_err(|e| SQLiteTransactionStorageError::DeleteError(e.to_string()))?;

        Ok(())
    }

    /// Delete pending transactions for a vault
    /// Equivalent to Swift's deletePendingTxs
    pub fn delete_pending_txs(&self, vault: &str) -> Result<(), SQLiteTransactionStorageError> {
        self.db
            .execute(
                "DELETE FROM transactions WHERE address = ?1 AND status = ?2",
                params![vault, tx_status::PENDING],
            )
            .map_err(|e| SQLiteTransactionStorageError::DeleteError(e.to_string()))?;

        Ok(())
    }
}

/// Errors that can occur during transaction storage operations
#[derive(Debug, thiserror::Error)]
pub enum SQLiteTransactionStorageError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Path error: {0}")]
    PathError(String),
    #[error("Schema creation failed for table {0}: {1}")]
    SchemaCreationFailed(String, String),
    #[error("Insert error: {0}")]
    InsertError(String),
    #[error("Query error: {0}")]
    QueryError(String),
    #[error("Delete error: {0}")]
    DeleteError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

// No Default impl: use SQLiteTransactionStorage::new() and handle Result to avoid panic on init failure.
