# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
# Build
cargo build
cargo build --release

# Test
cargo test
cargo test --package mfind-core  # Test specific crate

# Run CLI
cargo run -- --help
cargo run -- search <pattern>

# Lint
cargo clippy
cargo fmt
```

## Architecture Overview

### Workspace Structure

Rust workspace with 3 crates:

| Crate | Purpose |
|-------|---------|
| **mfind-core** | Core indexing and search engine (FST-based, filesystem scanning, query parsing) |
| **mfind-cli** | Command-line interface using clap |
| **mfind-tui** | Terminal UI (planned, ratatui-based) |

### mfind-core Modules

- **index/** - FST index engine, inode mapping, metadata cache
- **query/** - Query AST, parser, pattern matching, executor
- **fs/** - Filesystem scanner (ignore crate), FSEvents monitor
- **storage/** - Persistence layer trait + implementations
- **event/** - Filesystem event batching and deduplication
- **util/** - Path normalization, formatting utilities

### Key Design Decisions

1. **FST-based indexing** - Uses `fst` crate for memory-efficient string storage with O(m) prefix search
2. **Immutable FST** - FST structures are immutable; updates require rebuild
3. **Inode tracking** - DashMap for thread-safe inode → path mapping
4. **Async-first** - tokio for async operations, channels for event streaming
5. **Thread-safe reads** - DashMap and Arc-based sharing for concurrent access

### Data Flow

```
Scanner → FST Build → IndexEngine → Query Executor → Results
   ↑                                           ↓
FSEvents ←─────────── Index Update ←───────── Pattern Match
```

## Common Development Patterns

### Adding new query types
1. Add variant to `QueryNode` enum in `query/ast.rs`
2. Implement parsing in `query/parser.rs`
3. Add execution logic in `query/executor.rs`

### IndexEngine trait
Core operations defined in `index/engine.rs`:
- `build()` - Build index from filesystem roots
- `search()` - Execute queries against index
- `update()` - Incremental updates from FSEvents

### FST Index constraints
- Keys must be inserted in **lexicographic order**
- Use `Set::new(data)` for deserialization (requires `Vec<u8>` not `&[u8]`)
- Stream iteration requires `Streamer` trait in scope

## Configuration

- Config file: `~/.config/mfind/config.toml`
- Uses `toml` crate for parsing
- Config struct in `crates/mfind-cli/src/config/mod.rs`

## Release Optimization

```toml
[profile.release]
lto = true
codegen-units = 1
panic = "abort"
strip = true
```
