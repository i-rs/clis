# i-rs-walkdog

Dog walking tracking CLI tool for recording when you walk your dog.

## Features

- Track walking duration
- Different dogs support
- Time-based history
- Tag support for categorization

## Install

```bash
npm install -g @i-rs/i-rs-walkdog
# or
brew install i-rs/homebrew-tap/i-rs-walkdog
```

## Quick Start

```bash
# Record dog walk
i-rs-walkdog add "Buddy" 30
i-rs-walkdog add "Max" 45 --tag morning

# List all records
i-rs-walkdog list

# Get record details
i-rs-walkdog get abc12345
```

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/walkdog.json`
- Linux: `~/.config/i-rs/walkdog.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0