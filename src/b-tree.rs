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