/// This file contains implentation details of B Tree indexing structure for the database reads.

use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tokio::sync::RwLock;
use std::sync::Arc;
use async_recursion::async_recursion;

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
    root: Arc<RwLock<Option<BTreeNodeId>>>,
    nodes: Arc<RwLock<BTreeMap<BTreeNodeId, BTreeNode>>>,
    next_node_id: Arc<RwLock<BTreeNodeId>>,
    config: DatabaseConfig,
}

impl BTree {
    pub fn new(config: DatabaseConfig) -> Self {
        Self {
            root: Arc::new(RwLock::new(None)),
            nodes: Arc::new(RwLock::new(BTreeMap::new())),
            next_node_id: Arc::new(RwLock::new(0)),
            config,
        }
    }

    pub async fn get(&self, key: &Key) -> DatabaseResult<Option<Entry>> {
        let root_id = self.root.read().await;
        if let Some(root_id) = *root_id {
            self.get_from_node(root_id, key).await
        } else {
            Ok(None)
        }
    }

    #[async_recursion]
    async fn get_from_node(&self, node_id: BTreeNodeId, key: &Key) -> DatabaseResult<Option<Entry>> {
        let child_to_search = {
            let nodes = self.nodes.read().await;
            if let Some(node) = nodes.get(&node_id) {
                let mut child_to_search = None;
                for (i, node_key) in node.keys.iter().enumerate() {
                    match key.cmp(node_key) {
                        std::cmp::Ordering::Equal => {
                            if let Some(value) = &node.values[i] {
                                return Ok(Some(Entry::new_put(
                                    key.clone(),
                                    value.clone(),
                                    OperationId::new(0, 0),
                                )));
                            }
                            return Ok(None);
                        }
                        std::cmp::Ordering::Less => {
                            if !node.is_leaf && i < node.children.len() {
                                if let Some(child_id) = node.children[i] {
                                    child_to_search = Some(child_id);
                                    break;
                                }
                            }
                            return Ok(None);
                        }
                        std::cmp::Ordering::Greater => continue,
                    }
                }

                if child_to_search.is_none() && !node.is_leaf && !node.children.is_empty() {
                    child_to_search = node.children.last().unwrap().clone();
                }
                child_to_search
            } else {
                None
            }
        };
        
        if let Some(child_id) = child_to_search {
            return self.get_from_node(child_id, key).await;
        }
        
        Ok(None)
    }

    pub async fn put(&self, key: Key, value: Value, operation_id: OperationId) -> DatabaseResult<()> {
        let mut root_guard = self.root.write().await;
        if root_guard.is_none() {
            let root_id = self.create_leaf_node().await;
            *root_guard = Some(root_id);
        }
        let root_id = root_guard.unwrap();
        drop(root_guard);
        
        self.insert_into_node(root_id, key, value, operation_id).await
    }

    pub async fn delete(&self, key: Key, operation_id: OperationId) -> DatabaseResult<()> {
        let root_guard = self.root.read().await;
        if let Some(root_id) = *root_guard {
            drop(root_guard);
            self.delete_from_node(root_id, key, operation_id).await
        } else {
            Ok(())
        }
    }

    async fn insert_into_node(&self, node_id: BTreeNodeId, key: Key, value: Value, _operation_id: OperationId) -> DatabaseResult<()> {
        let mut nodes = self.nodes.write().await;
        if let Some(node) = nodes.get_mut(&node_id) {
            if node.is_leaf {
                let pos = node.keys.binary_search(&key).unwrap_or_else(|x| x);
                node.keys.insert(pos, key);
                node.values.insert(pos, Some(value));
            }
        }
        Ok(())
    }

    async fn insert_into_leaf(&self, _node_id: BTreeNodeId, _key: Key, _value: Value, _operation_id: OperationId) -> DatabaseResult<()> {
        Ok(())
    }

    async fn delete_from_node(&self, _node_id: BTreeNodeId, _key: Key, _operation_id: OperationId) -> DatabaseResult<()> {
        Ok(())
    }

    async fn split_node(&self, _node_id: BTreeNodeId) -> DatabaseResult<()> {
        Ok(())
    }

    async fn create_leaf_node(&self) -> BTreeNodeId {
        let mut next_id = self.next_node_id.write().await;
        let node_id = *next_id;
        *next_id += 1;
        
        let mut nodes = self.nodes.write().await;
        let node = BTreeNode {
            keys: Vec::new(),
            values: Vec::new(),
            children: Vec::new(),
            is_leaf: true,
            size: 0,
        };
        nodes.insert(node_id, node);
        node_id
    }

    pub async fn scan(&self, _start_key: Option<Key>, _end_key: Option<Key>) -> DatabaseResult<Vec<Entry>> {
        Ok(Vec::new())
    }

    pub async fn size(&self) -> usize {
        self.nodes.read().await.len()
    }
}