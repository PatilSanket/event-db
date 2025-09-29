use lsm_btree_db::{Database, DatabaseBuilder, DatabaseConfig};
use std::path::PathBuf;
use std::env;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting LSM-BTree Database Server");

    let db_path = env::var("DATABASE_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp/lsm_btree_db"));

    info!("Database path: {:?}", db_path);

    let config = DatabaseConfig {
        max_memtable_size: 16 * 1024 * 1024,
        max_sstable_size: 64 * 1024 * 1024,
        max_levels: 7,
        compaction_threshold: 4,
        btree_node_size: 8192,
    };

    let db_config = DatabaseBuilder::new()
        .with_config(config)
        .with_base_path(db_path)
        .build()
        .await?;

    info!("Database initialized successfully");
    let db = std::sync::Arc::new(db_config);

    /// Health check and teardown logic
    /// Benchmarking logic
    /// Sample read-write-eventSToring logic 
    
    db.close().await?;
    info!("Database closed successfully");

    Ok(())
}