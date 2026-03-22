# mfind Development Guide

## Architecture Overview

mfind uses a layered architecture:

```
┌─────────────────────────────────────────┐
│           Interface Layer               │
│  CLI (mfind-cli)  │  TUI (mfind-tui)    │
└─────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────┐
│           Service Layer                 │
│  SearchService │ IndexService │ Monitor │
└─────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────┐
│           Core Engine                   │
│  IndexEngine │ QueryEngine │ Storage    │
└─────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────┐
│         Filesystem Abstraction          │
│  Scanner │ Monitor │ Backend            │
└─────────────────────────────────────────┘
```

## Project Structure

```
mfind/
├── crates/
│   ├── mfind-core/    # Core library (index, query, fs)
│   ├── mfind-cli/     # Command-line interface
│   └── mfind-tui/     # Terminal UI (future)
├── tests/
│   ├── integration/   # Integration tests
│   └── benchmarks/    # Benchmark tests
└── docs/              # Documentation
```

## Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Build specific crate
cargo build -p mfind-core
```

## Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_search_prefix

# Run benchmarks
cargo bench
```

## Code Style

### Import Order

1. Standard library
2. External crates
3. Internal modules
4. Parent/sibling modules

```rust
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use tokio::fs;

use crate::index::IndexEngine;
use crate::query::Query;
```

### Error Handling

Use `anyhow::Result` for application code, `thiserror` for libraries:

```rust
// Library code
#[derive(thiserror::Error, Debug)]
pub enum IndexError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// Application code
pub async fn build_index() -> anyhow::Result<()> {
    // ...
}
```

### Async Code

Use `tokio` for async runtime:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Async code here
}
```

## Adding New Features

1. Create feature branch: `git checkout -b feature/my-feature`
2. Implement in `mfind-core` first
3. Add CLI/TUI bindings
4. Write tests
5. Update documentation
6. Create PR

## Performance Guidelines

1. **Use FST for string storage** - Memory efficient
2. **Parallel iteration** - Use `rayon` for CPU-bound work
3. **Batch operations** - Minimize lock contention
4. **Avoid allocations** - Reuse buffers when possible

## Debugging

Enable verbose logging:

```bash
RUST_LOG=mfind=debug cargo run -- search "pattern"
```

## Releasing

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Run tests: `cargo test`
4. Build: `cargo build --release`
5. Tag: `git tag -a v0.1.0 -m "Release v0.1.0"`
6. Push: `git push origin --tags`
