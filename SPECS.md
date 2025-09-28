## API Specification

### Core Operations

#### Write Operations

```rust
// Put a key-value pair
pub async fn put(&self, key: Key, value: Value) -> DatabaseResult<OperationId>

// Delete a key
pub async fn delete(&self, key: Key) -> DatabaseResult<OperationId>

// Batch operations
pub async fn batch_put(&self, operations: Vec<(Key, Value)>) -> DatabaseResult<Vec<OperationId>>
pub async fn batch_delete(&self, keys: Vec<Key>) -> DatabaseResult<Vec<OperationId>>
```

#### Read Operations

```rust
// Get a single value
pub async fn get(&self, key: &Key) -> DatabaseResult<Option<Entry>>

// Range scan
pub async fn scan(&self, start_key: Option<Key>, end_key: Option<Key>) -> DatabaseResult<Vec<Entry>>

```

#### Management Operations

```rust
// Flush data to disk
pub async fn flush(&self) -> DatabaseResult<()>

// Compact storage
pub async fn compact(&self) -> DatabaseResult<()>

// Get statistics
pub async fn stats(&self) -> DatabaseResult<DatabaseStats>

// Close database
pub async fn close(&self) -> DatabaseResult<()>
```

### Event Operations

```rust
// Get all events
pub async fn get_events(&self) -> DatabaseResult<Vec<DatabaseEvent>>

// Get events since operation
pub async fn get_events_since(&self, operation_id: OperationId) -> DatabaseResult<Vec<DatabaseEvent>>

// Subscribe to events
pub async fn subscribe(&self, name: String, handler: Box<dyn EventHandler>)

// Unsubscribe from events
pub async fn unsubscribe(&self, name: &str)
```

## Data Models

### Core Types

```rust
// Key type
pub type Key = Vec<u8>;

// Value type
pub type Value = Vec<u8>;

// Timestamp for ordering
pub type Timestamp = u64;

// Sequence number
pub type SequenceNumber = u64;

// Operation identifier
pub struct OperationId {
    pub timestamp: Timestamp,
    pub sequence: SequenceNumber,
}
```

### Entry Model

```rust
pub struct Entry {
    pub key: Key,
    pub value: Option<Value>,  // None for delete operations
    pub operation_id: OperationId,
    pub is_deleted: bool,
}
```

### Operation Model

```rust
pub enum Operation {
    Put { key: Key, value: Value },
    Delete { key: Key },
}
```

### Event Model

```rust
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
```