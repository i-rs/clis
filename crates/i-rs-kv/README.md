# i-rs-kv

Key-value storage CLI tool for storing and retrieving arbitrary data.

## Features

- Simple key-value storage
- Tag support for categorization
- Time-based metadata
- Quick get/set operations
- JSON output support

## Install

```bash
npm install -g @i-rs/i-rs-kv
# or
brew install i-rs/homebrew-tap/i-rs-kv
```

## Quick Start

```bash
# Set a value
i-rs-kv add username john
i-rs-kv add api-url "https://api.example.com" --tag config

# Get a value
i-rs-kv get username

# List all entries
i-rs-kv list

# List with JSON output
i-rs-kv list --json
```

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/kv.json`
- Linux: `~/.config/i-rs/kv.json`
- Windows: `~\AppData\Roaming\i-rs\kv.json`

## License

MIT OR Apache-2.0