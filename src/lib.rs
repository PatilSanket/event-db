pub mod types;
pub mod error;
pub mod database;
pub mod wal;

// Use underscore names for file modules with hyphens
#[path = "b-tree.rs"]
pub mod b_tree;

#[path = "lsm-tree.rs"] 
pub mod lsm_tree;

// Re-export main types
pub use database::{Database, DatabaseBuilder};
pub use types::{DatabaseConfig, Key, Value, Entry, OperationId};
pub use error::DatabaseError;
pub use wal::WriteAheadLog;
