# mfind

[![Crates.io](https://img.shields.io/crates/v/mfind.svg)](https://crates.io/crates/mfind)
[![License](https://img.shields.io/crates/l/mfind)](LICENSE)
[![Build Status](https://img.shields.io/github/actions/workflow/status/mfind/mfind/ci.yml)](https://github.com/mfind/mfind/actions)

**Fast, independent file search for macOS** - A modern file search tool inspired by Windows Everything and fd.

## Features

- ⚡ **Lightning fast** - FST-based indexing for sub-millisecond search
- 🔍 **Real-time sync** - FSEvents monitoring for instant updates
- 🍎 **macOS native** - Optimized for APFS filesystem
- 🎯 **Developer friendly** - .gitignore support, JSON output, CLI-first design
- 🆓 **Independent** - Does not rely on Spotlight
- 📦 **Open source** - MIT/Apache 2.0 licensed

## Installation

### From Source

```bash
git clone https://github.com/mfind/mfind.git
cd mfind
cargo build --release
cargo install --path .
```

### Homebrew (Coming Soon)

```bash
brew install mfind
```

## Quick Start

```bash
# Build your first index
mfind index build ~/Documents ~/Downloads

# Search for files
mfind apps              # Prefix search
mfind "*.pdf"           # Wildcard search
mfind --regex ".*\.rs$" # Regex search
mfind -e rs             # By extension

# Advanced usage
mfind "config" --type file          # Only files
mfind "log" --size ">1MB"           # By size
mfind "test" --modified "today"     # By time
mfind "rust AND cargo"              # Boolean search
```

## Commands

### Search

```bash
mfind [OPTIONS] [PATTERN]

Options:
  -r, --regex         Use regular expression
  -s, --case-sensitive  Case sensitive search
  -p, --path <PATH>   Search in specific path
  -e, --ext <EXT>     Filter by extension
  -n, --limit <N>     Maximum results (default: 1000)
  -o, --output <FMT>  Output format: list, json, table
  --hidden            Include hidden files
  --no-gitignore      Don't respect .gitignore
  --type <TYPE>       Filter by type: file, dir, link
  -l, --long-list     Show detailed information
```

### Index Management

```bash
# Build index
mfind index build ~/Documents

# Show status
mfind index status

# Export/Import
mfind index export > backup.idx
mfind index import < backup.idx

# Clear index
mfind index clear
```

### Service Management

```bash
# Install background service
mfind service install

# Start/Stop
mfind service start
mfind service stop

# Check status
mfind service status
```

## Configuration

Configuration file location: `~/.config/mfind/config.toml`

```toml
[global]
memory_limit = 512
parallelism = 4
log_level = "info"

[index]
roots = ["~/Documents", "~/Downloads"]
exclude_dirs = ["node_modules", ".git", "target"]
gitignore = true
include_hidden = false

[search]
default_limit = 1000
highlight = true
```

## Roadmap

- [ ] **Phase 1 (MVP)**: Basic CLI search ✅
- [ ] **Phase 2**: FSEvents monitoring, TUI
- [ ] **Phase 3**: Background service, launchd integration
- [ ] **Phase 4**: macOS GUI (Tauri/SwiftUI)
- [ ] **Phase 5**: Cross-platform (Linux/Windows)

## Performance

| Metric | Target | Everything |
|--------|--------|------------|
| 1M file index | < 10s | 5-10s |
| Search (prefix) | < 50ms | < 50ms |
| Memory (1M files) | < 200MB | ~150MB |
| Real-time sync | < 500ms | < 1s |

## Comparison

| Feature | mfind | fd | Spotlight | Everything |
|---------|-------|----|-----------|------------|
| Index | ✅ FST | ❌ | ✅ System | ✅ MFT |
| Real-time | ✅ FSEvents | ❌ | ✅ | ✅ USN |
| Spotlight independent | ✅ | ✅ | ❌ | ✅ |
| CLI | ✅ | ✅ | ⚠️ | ⚠️ |
| GUI planned | ✅ | ❌ | ✅ | ✅ |
| Open source | ✅ | ✅ | ❌ | ❌ |

## Development

### Requirements

- Rust 1.68+
- macOS 11.0+

### Build

```bash
cargo build
cargo build --release
```

### Test

```bash
cargo test
cargo bench
```

### Generate Completions

```bash
mfind completions bash > /etc/bash_completion.d/mfind
mfind completions zsh > /usr/local/share/zsh/site-functions/_mfind
mfind completions fish > ~/.config/fish/completions/mfind.fish
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Acknowledgments

- [Everything](https://www.voidtools.com/) - Inspiration for fast file search
- [fd](https://github.com/sharkdp/fd) - Excellent CLI design
- [ripgrep](https://github.com/BurntSushi/ripgrep) - Performance optimization reference
