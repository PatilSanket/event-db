/// This file contains implentation details of B Tree indexing structure for the database reads.

use crate::types::*;
use crate::error::DatabaseError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BTreeNode {
    pub keys: Vec<Key>,
    pub values: Vec<Option<Value>>,
    pub children: Vec<Option<BTreeNodeId>>,
    pub is_leaf: bool,
    pub size: usize,
}

pub type BTreeNodeId = u64;

pub struct BTree {
    root: Option<BTreeNodeId>,
    nodes: BTreeMap<BTreeNodeId, BTreeNode>,
    next_node_id:BTreeNodeId>,
    config: DatabaseConfig,
}

impl BTree {
    pub fn new(config: DatabaseConfig) -> Self {
        /// Root with lock
        /// Nodes with lock
        /// next_node_id with lock
        /// I'm missing something else here TODO: Revise about locking strategies
    }

    pub async fn get(&self, key: &Key) -> DatabaseResult<Option<Entry>> {
        let root_id = self.root.read().await;
        if let Some(root_id) = *root_id {
            self.get_from_node(root_id, key).await
        } else {
            Ok(None)
        }
    }

    pub async fn put(&self, key: Key, value: Value, operation_id: OperationId) -> DatabaseResult<()> {
    }

    pub async fn delete(&self, key: Key, operation_id: OperationId) -> DatabaseResult<()> {
    }

    async fn insert_into_node(&self, node_id: BTreeNodeId, key: Key, value: Value, operation_id: OperationId) -> DatabaseResult<()> {
    }

    async fn insert_into_leaf(&self, node_id: BTreeNodeId, key: Key, value: Value, operation_id: OperationId) -> DatabaseResult<()> {
    }

    async fn delete_from_node(&self, node_id: BTreeNodeId, key: Key, operation_id: OperationId) -> DatabaseResult<()> {
    }

    async fn split_node(&self, node_id: BTreeNodeId) -> DatabaseResult<()> {
    }

    async fn create_leaf_node(&self) -> BTreeNodeId {
    }

    pub async fn scan(&self, start_key: Option<Key>, end_key: Option<Key>) -> DatabaseResult<Vec<Entry>> {
    }

    pub async fn size(&self) -> usize {
        self.nodes.read().await.len()
    }
}