# mfind Architecture

## Core Components

### 1. Index Engine

The index engine is responsible for building and maintaining the file index.

**Key components:**
- **FST Index**: Memory-efficient string storage using Finite State Transducers
- **Inode Map**: Maps inode numbers to file paths
- **Meta Cache**: LRU cache for file metadata

**Data flow:**
```
FileSystem → Scanner → FST Builder → Index
                      ↓
                  Inode Map
                      ↓
                  Meta Cache
```

### 2. Query Engine

Parses and executes search queries.

**Query types:**
- **Prefix**: `mfind "app"` - Fast O(m) lookup
- **Wildcard**: `mfind "*.txt"` - Converted to regex
- **Regex**: `mfind --regex ".*\.rs$"` - Full regex
- **Boolean**: `mfind "rust AND cargo"` - Set operations

### 3. Filesystem Layer

Abstracts filesystem operations.

**Components:**
- **Scanner**: Parallel directory traversal
- **Monitor**: FSEvents-based real-time monitoring
- **Backend**: Filesystem type detection

### 4. Storage Layer

Persists index data.

**Implementations:**
- **Memory**: In-memory storage (default)
- **LMDB**: Disk-backed storage (future)

## Data Structures

### FST Index

```rust
pub struct FSTIndex {
    set: Set<Vec<u8>>,  // fst crate
}
```

**Properties:**
- O(m) prefix search where m is pattern length
- Highly compressed (5-10x better than HashMap)
- Immutable (rebuild on changes)

### Inode Map

```rust
pub struct InodeMap {
    map: DashMap<u64, PathBuf>,
}
```

**Properties:**
- O(1) lookup
- Thread-safe ( DashMap)
- Maps inode → path

## Event Flow

```
FSEvents → Event Batch → Dedup → Index Update
   ↓
Event Stream → Application
```

## Thread Model

```
Main Thread (CLI/TUI)
    ↓ (async)
Search Service
    ↓
Index Engine (read-only)
    ↓
FST + Inode Map (concurrent read)

Monitor Thread
    ↓ (channel)
Event Processor
    ↓ (write lock)
Index Engine (write)
```

## Performance Characteristics

| Operation | Complexity | Notes |
|-----------|------------|-------|
| Prefix search | O(m) | m = pattern length |
| Regex search | O(n × m) | n = entries, m = pattern |
| Index build | O(n log n) | Sorting dominated |
| Event update | O(1) | Single entry |
| Memory usage | O(n × k) | k = avg path length |

## Future Extensions

1. **Content Indexing**: Full-text search support
2. **Distributed Index**: Network volume support
3. **Machine Learning**: Relevance ranking
4. **Plugin System**: Extensible search filters
