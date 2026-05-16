# i-rs-kv

Key-value storage CLI tool for storing and retrieving arbitrary data.

## Features

- Simple key-value storage
- Tag support for categorization
- Time-based metadata
- Quick get/set operations
- JSON output support
- Data export/import/clear

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

## Commands

| Command | Description |
|---------|-------------|
| `add <KEY> <VALUE>` | Add a new key-value entry |
| `get <KEY>` | Get entry details |
| `list [--tag]` | List all entries |
| `update <KEY> [--value] [--tag] [--remark]` | Update an entry |
| `delete <KEY>` | Delete an entry |
| `search <QUERY>` | Search entries by value content |
| `stats` | Show entry statistics |
| `copy <SRC> <DST>` | Copy an entry |
| `rename <OLD> <NEW>` | Rename an entry |
| `data {export|import|clear}` | Manage stored data |
| `example` | Show usage examples |
| `skill [sub]` | Show skill information |

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/kv.json`
- Linux: `~/.config/i-rs/kv.json`
- Windows: `~\AppData\Roaming\i-rs\kv.json`

## License

MIT OR Apache-2.0