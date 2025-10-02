/// This file contains implentation details of LSM Tree indexing structure for the database writes.
use crate::types::*;
use crate::error::DatabaseError;
use lsm_tree::{Config, Tree, AbstractTree};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct LSMTree {
    tree: Tree,
    sequence_counter: Arc<RwLock<SequenceNumber>>,
}

impl LSMTree {
    pub fn new(path: std::path::PathBuf) -> DatabaseResult<Self> {
        std::fs::create_dir_all(&path)?;
        
        let config = Config::new(path);
        let tree = config.open().map_err(|e| DatabaseError::InvalidOperation(e.to_string()))?;
        
        Ok(Self {
            tree,
            sequence_counter: Arc::new(RwLock::new(0)),
        })
    }
    
    pub async fn put(&self, key: Key, value: Value) -> DatabaseResult<OperationId> {
        let mut seq = self.sequence_counter.write().await;
        *seq += 1;
        let operation_id = OperationId::new(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            *seq,
        );
        
        let _ = self.tree.insert(&key, &value, *seq);
        
        Ok(operation_id)
    }
    
    pub async fn get(&self, key: &Key) -> DatabaseResult<Option<Value>> {
        match self.tree.get(key, None) {
            Ok(Some(value)) => Ok(Some(value.to_vec())),
            Ok(None) => Ok(None),
            Err(e) => Err(DatabaseError::InvalidOperation(e.to_string())),
        }
    }
    
    pub async fn delete(&self, key: Key) -> DatabaseResult<OperationId> {
        let mut seq = self.sequence_counter.write().await;
        *seq += 1;
        let operation_id = OperationId::new(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            *seq,
        );
        
        let _ = self.tree.remove(&key, *seq);
        
        Ok(operation_id)
    }
    
    pub async fn flush(&self) -> DatabaseResult<()> {
        let seq = *self.sequence_counter.read().await;
        self.tree.flush_active_memtable(seq).map_err(|e| DatabaseError::InvalidOperation(e.to_string()))?;
        Ok(())
    }
}