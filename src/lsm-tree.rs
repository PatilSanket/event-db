/// This file contains implentation details of LSM Tree indexing structure for the database writes.
use crate::types::*;
use crate::error::DatabaseError;
use lsm_tree::{Config, LsmTree};
use serde::{Deserialize, Serialize};

/// LSM Tree wrapper using the lsm-tree crate
pub struct LSMTree {
    tree: LsmTree,
    sequence_counter: <SequenceNumber>,
}