use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// A key in the database
pub type Key = Vec<u8>;

/// A value in the database
pub type Value = Vec<u8>;

/// Timestamp for ordering operations
pub type Timestamp = u64;

/// Sequence number for ordering within the same timestamp
pub type SequenceNumber = u64;

/// A unique identifier for an operation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OperationId {
    pub timestamp: Timestamp,
    pub sequence: SequenceNumber,
}

impl OperationId {
    pub fn new(timestamp: Timestamp, sequence: SequenceNumber) -> Self {
        Self { timestamp, sequence }
    }
}

impl PartialOrd for OperationId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OperationId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.timestamp
            .cmp(&other.timestamp)
            .then(self.sequence.cmp(&other.sequence))
    }
}

/// Database operation types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
    Put { key: Key, value: Value },
    Delete { key: Key },
}

/// An entry in the database with metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub key: Key,
    pub value: Option<Value>, // None for delete operations
    pub operation_id: OperationId,
    pub is_deleted: bool,
}

impl Entry {
    pub fn new_put(key: Key, value: Value, operation_id: OperationId) -> Self {
        Self {
            key,
            value: Some(value),
            operation_id,
            is_deleted: false,
        }
    }

    pub fn new_delete(key: Key, operation_id: OperationId) -> Self {
        Self {
            key,
            value: None,
            operation_id,
            is_deleted: true,
        }
    }
}

/// Event types for the CQRS pattern
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DatabaseEvent {
    WriteEvent {
        operation: Operation,
        operation_id: OperationId,
    },
    ReadEvent {
        key: Key,
        operation_id: OperationId,
    },
    CompactionEvent {
        level: usize,
        operation_id: OperationId,
    },
}

/// Configuration for the database
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub max_memtable_size: usize,
    pub max_sstable_size: usize,
    pub max_levels: usize,
    pub compaction_threshold: usize,
    pub btree_node_size: usize,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            max_memtable_size: 1024 * 1024, // 1MB
            max_sstable_size: 10 * 1024 * 1024, // 10MB
            max_levels: 7,
            compaction_threshold: 4,
            btree_node_size: 4096,
        }
    }
}

/// Result type for database operations
pub type DatabaseResult<T> = Result<T, crate::error::DatabaseError>;