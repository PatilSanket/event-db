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

    /*
    // Test basic read/write functionality
    info!("Testing basic database operations...");
    
    // Test write operations
    let test_key = b"test_key".to_vec();
    let test_value = b"test_value".to_vec();
    
    info!("Writing test data...");
    db.put(test_key.clone(), test_value.clone()).await?;
    
    // Test read operations
    info!("Reading test data...");
    match db.get(&test_key).await? {
        Some(value) => {
            if value == test_value {
                info!("✓ Read/Write test passed!");
            } else {
                info!("✗ Read/Write test failed: value mismatch");
            }
        }
        None => {
            info!("✗ Read/Write test failed: key not found");
        }
    }
    
    // Test another key-value pair
    let key2 = b"hello".to_vec();
    let value2 = b"world".to_vec();
    
    db.put(key2.clone(), value2.clone()).await?;
    
    if let Some(retrieved) = db.get(&key2).await? {
        info!("✓ Second test passed: {} = {}", 
              String::from_utf8_lossy(&key2), 
              String::from_utf8_lossy(&retrieved));
    }
    
    info!("Database operations completed successfully");
    */

    println!("Welcome to Event DB Interactive Mode");
    println!("Available commands:");
    println!("  put <key> <value>  - Write data");
    println!("  get <key>          - Read data");
    println!("  delete <key>       - Delete data");
    println!("  exit               - Exit program");
    println!();

    let stdin = std::io::stdin();
    let mut input = String::new();

    loop {
        input.clear();
        print!("> ");
        use std::io::Write;
        std::io::stdout().flush()?;

        if stdin.read_line(&mut input)? == 0 {
            break;
        }

        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "put" => {
                if parts.len() < 3 {
                    println!("Usage: put <key> <value>");
                    continue;
                }
                let key = parts[1].as_bytes().to_vec();
                let value = parts[2].as_bytes().to_vec();
                match db.put(key, value).await {
                    Ok(_) => println!("OK"),
                    Err(e) => println!("Error: {}", e),
                }
            }
            "get" => {
                if parts.len() < 2 {
                    println!("Usage: get <key>");
                    continue;
                }
                let key = parts[1].as_bytes().to_vec();
                match db.get(&key).await {
                    Ok(Some(value)) => println!("{}", String::from_utf8_lossy(&value)),
                    Ok(None) => println!("(not found)"),
                    Err(e) => println!("Error: {}", e),
                }
            }
            "delete" => {
                if parts.len() < 2 {
                    println!("Usage: delete <key>");
                    continue;
                }
                let key = parts[1].as_bytes().to_vec();
                match db.delete(key).await {
                    Ok(_) => println!("OK"),
                    Err(e) => println!("Error: {}", e),
                }
            }
            "exit" | "quit" => break,
            "help" => {
                println!("Available commands:");
                println!("  put <key> <value>");
                println!("  get <key>");
                println!("  delete <key> (Not yet functional)");
                println!("  exit");
            }
            _ => println!("Unknown command. Type 'help' for usage."),
        }
    }
    
    info!("Closing database...");
    db.close().await?;
    info!("Database closed successfully");

    Ok(())
}