# event-db
A database which uses LSM-Tree for writes, B-Tree for reads and event sourcing pattern to communicate between the both. Additionally it (will) support(s) generic Capture Data Change (CDC) mechanism.


## Data Flow Architecture

### Write Path
```
Client Request
      │
      ▼
┌─────────────┐
│ Database API│
└─────────────┘
      │
      ▼
┌─────────────┐    ┌─────────────┐
│ Command     │───▶│ LSM Tree    │
│ Handler     │    │ (Writes)    │
└─────────────┘    └─────────────┘
      │                   │
      ▼                   ▼
┌─────────────┐    ┌─────────────┐
│ Event       │    │ MemTable    │
│ Stream      │    │             │
└─────────────┘    └─────────────┘
      │                   │
      ▼                   ▼
┌─────────────┐    ┌─────────────┐
│ B-Tree      │    │ SSTables    │
│ Projection  │    │ (On Disk)   │
└─────────────┘    └─────────────┘
```

### Read Path
```
Client Request
      │
      ▼
┌─────────────┐
│ Database API│
└─────────────┘
      │
      ▼
┌─────────────┐
│ Query       │
│ Handler     │
└─────────────┘
      │
      ▼
┌─────────────┐   
│ B-Tree      │   
└─────────────┘    
      │                  
      ▼                
┌─────────────┐    
│ Result      │   
└─────────────┘    
```

## Navigating File Structure

### The Coordinator (database.rs)

This is the main entry point. Currently, it performs a synchronous dual-write:

Write Path: When you call put(), it first writes to the LSM Tree.

Read Path: Immediately after the LSM write succeeds, it directly calls self.btree.put() to update the B-Tree.

### Write Path (lsm-tree.rs)

The lsm_tree crate is used for the heavy lifting.

It functions as the primary persistent store.

### Read Path (b-tree.rs)

This is a custom in-memory B-Tree implementation.

It uses a BTreeMap<BTreeNodeId, BTreeNode> to store nodes. This models the B-Tree structure (keys, values, children) explicitly in memory.

It acts as a "Read View". The get() method tries to find the key here first.

If the key isn't found in the B-Tree, it falls back to the LSM Tree (self.lsm_tree.get(key) in database.rs).

## TODO:
1. Decouple Writes: instead of calling btree.put directly in database.rs, we would want to push the Operation to an internal channel or log.
2. Implement the WAL/Stream: We need a way to capture the "events" (like Put, Delete) into a persistent log that supports replay.
3. Create a Consumer: A background task that reads from this log and applies updates to the B-Tree asynchronously.