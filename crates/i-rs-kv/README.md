# i-rs-kv

Key-value storage CLI tool for storing and retrieving arbitrary data.

## Features

- Simple key-value storage
- Tag support for categorization
- Time-based metadata
- Quick get/set operations

## Install

```bash
npm install -g @i-rs/i-rs-kv
# or
brew install i-rs/homebrew-tap/i-rs-kv
```

## Quick Start

```bash
# Set a value
i-rs-kv add "username" --value "john"
i-rs-kv add "api-url" --value "https://api.example.com"

# Get a value
i-rs-kv get username

# List all entries
i-rs-kv list
```

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/kv.json`
- Linux: `~/.config/i-rs/kv.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0