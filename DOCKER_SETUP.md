
### Using Docker Compose
```bash
# Build and run the database
docker-compose up --build

# Run in detached mode
docker-compose up -d --build

# View logs
docker-compose logs -f

# Stop the database
docker-compose down
```

### Using Docker directly
```bash
# Build the image
docker build -t event-db .

# Run the container
docker run -v $(pwd)/data:/data -e DATABASE_PATH=/data event-db

# Run with custom database path
docker run -v /your/data/path:/data -e DATABASE_PATH=/data event-db
```

## Environment Variables

- `DATABASE_PATH`: Path where the database files will be stored (default: `/data`)
- `RUST_LOG`: Log level (default: `info`, options: `trace`, `debug`, `info`, `warn`, `error`)

## Features

This minimal implementation provides:

1. **LSM Tree**: For efficient writes and storage
2. **B-Tree**: For fast reads 
3. **Basic Operations**: PUT, GET, DELETE
4. **Docker Support**: Containerized deployment
5. **Persistent Storage**: Data persists in mounted volumes

## Testing

The application runs basic tests on startup to verify:
- Write operations to both LSM tree and B-tree
- Read operations from both storage engines
- Data consistency between operations

Look for "✓" messages in the logs to confirm successful operations.
