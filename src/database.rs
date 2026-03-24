/// Main database implementation with LSM tree for writes and B-Tree for reads
use crate::types::*;
use crate::b_tree::BTree;
use crate::lsm_tree::LSMTree;
use crate::wal::{WriteAheadLog, WalEntry};
use std::path::PathBuf;
use tracing::info;

pub struct Database {
    lsm_tree: LSMTree,
    btree: BTree,
    wal: Option<WriteAheadLog>,
    config: DatabaseConfig,
}

impl Database {
    pub async fn new(path: PathBuf, config: DatabaseConfig) -> DatabaseResult<Self> {
        let lsm_tree = LSMTree::new(path.clone())?;
        let btree = BTree::new(config.clone());
        
        // Open the WAL if enabled.
        let wal = if config.wal_enabled {
            let wal_path = path.join("wal.log");

            // Recover any unflushed entries before we accept new writes.
            let entries = WriteAheadLog::recover(&wal_path)?;
            if !entries.is_empty() {
                info!("WAL recovery: replaying {} entries", entries.len());
                Self::replay_wal_entries(&lsm_tree, &btree, &entries).await?;
            }

            Some(WriteAheadLog::open(wal_path, config.wal_sync_on_write)?)
        } else {
            None
        };
        
        Ok(Self {
            lsm_tree,
            btree,
            wal,
            config,
        })
    }
    
    pub async fn put(&self, key: Key, value: Value) -> DatabaseResult<()> {
        // WAL first — ensure the operation is durable before touching in-memory state.
        if let Some(wal) = &self.wal {
            let op = Operation::Put { key: key.clone(), value: value.clone() };
            let seq = self.lsm_tree.next_sequence().await;
            wal.append(&op, seq).await?;
        }

        let operation_id = self.lsm_tree.put(key.clone(), value.clone()).await?;
        self.btree.put(key, value, operation_id).await?;
        
        Ok(())
    }
    
    pub async fn get(&self, key: &Key) -> DatabaseResult<Option<Value>> {
        if let Some(entry) = self.btree.get(key).await? {
            if !entry.is_deleted {
                return Ok(entry.value);
            }
        }
        
        self.lsm_tree.get(key).await
    }
    
    pub async fn delete(&self, key: Key) -> DatabaseResult<()> {
        // WAL first.
        if let Some(wal) = &self.wal {
            let op = Operation::Delete { key: key.clone() };
            let seq = self.lsm_tree.next_sequence().await;
            wal.append(&op, seq).await?;
        }

        let operation_id = self.lsm_tree.delete(key.clone()).await?;
        self.btree.delete(key, operation_id).await?;
        
        Ok(())
    }
    
    pub async fn close(&self) -> DatabaseResult<()> {
        self.lsm_tree.flush().await?;
        
        // After a successful flush the WAL can be truncated — all data is
        // safely persisted in the LSM tree's SSTables.
        if let Some(wal) = &self.wal {
            wal.truncate().await?;
        }
        
        Ok(())
    }

    // ── Internal helpers ───────────────────────────────────────────────

    /// Replay recovered WAL entries into the LSM tree and B-tree.
    async fn replay_wal_entries(
        lsm_tree: &LSMTree,
        btree: &BTree,
        entries: &[WalEntry],
    ) -> DatabaseResult<()> {
        for entry in entries {
            match &entry.operation {
                Operation::Put { key, value } => {
                    let op_id = lsm_tree.put(key.clone(), value.clone()).await?;
                    btree.put(key.clone(), value.clone(), op_id).await?;
                }
                Operation::Delete { key } => {
                    let op_id = lsm_tree.delete(key.clone()).await?;
                    btree.delete(key.clone(), op_id).await?;
                }
            }
        }
        Ok(())
    }
}

pub struct DatabaseBuilder {
    config: Option<DatabaseConfig>,
    base_path: Option<PathBuf>,
}

impl DatabaseBuilder {
    pub fn new() -> Self {
        Self {
            config: None,
            base_path: None,
        }
    }
    
    pub fn with_config(mut self, config: DatabaseConfig) -> Self {
        self.config = Some(config);
        self
    }
    
    pub fn with_base_path(mut self, path: PathBuf) -> Self {
        self.base_path = Some(path);
        self
    }
    
    pub async fn build(self) -> DatabaseResult<Database> {
        let config = self.config.unwrap_or_default();
        let path = self.base_path.unwrap_or_else(|| PathBuf::from("/tmp/lsm_btree_db"));
        
        Database::new(path, config).await
    }
}