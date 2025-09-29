# event-db
A database which uses LSM-Tree for writes, B-Tree for reads and event sourcing pattern to communicate between the both. Additionally it supports generic Capture Data Change (CDC) mechanism.


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
