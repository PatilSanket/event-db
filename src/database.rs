/// Main database implementation with LSM tree for writes and B-Tree for reads
use crate::types::*;
use crate::error::DatabaseError;
use crate::b_tree::BTree;
use crate::lsm_tree::LSMTree;
use std::path::PathBuf;
use std::sync::Arc;

pub struct Database {
    lsm_tree: LSMTree,
    btree: BTree,
    config: DatabaseConfig,
}

impl Database {
    pub async fn new(path: PathBuf, config: DatabaseConfig) -> DatabaseResult<Self> {
        let lsm_tree = LSMTree::new(path)?;
        let btree = BTree::new(config.clone());
        
        Ok(Self {
            lsm_tree,
            btree,
            config,
        })
    }
    
    pub async fn put(&self, key: Key, value: Value) -> DatabaseResult<()> {
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
        let operation_id = self.lsm_tree.delete(key.clone()).await?;
        
        self.btree.delete(key, operation_id).await?;
        
        Ok(())
    }
    
    pub async fn close(&self) -> DatabaseResult<()> {
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