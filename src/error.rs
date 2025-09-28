use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Database is closed")]
    DatabaseClosed,
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
}