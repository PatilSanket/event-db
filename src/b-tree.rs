/// This file contains implentation details of B Tree indexing structure for the database reads.

use crate::types::*;
use crate::error::DatabaseError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// B-Tree node structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BTreeNode {
    pub keys: Vec<Key>,
    pub values: Vec<Option<Value>>, // None for deleted entries
    pub children: Vec<Option<BTreeNodeId>>,
    pub is_leaf: bool,
    pub size: usize,
}

/// Unique identifier for a B-Tree node
pub type BTreeNodeId = u64;

/// B-Tree implementation optimized for read operations
pub struct BTree {
    root: Option<BTreeNodeId>,
    nodes: BTreeMap<BTreeNodeId, BTreeNode>,
    next_node_id:BTreeNodeId>,
    config: DatabaseConfig,
}