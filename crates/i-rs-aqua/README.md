# i-rs-aqua

Aquarium water change tracking CLI tool for recording when you change fish tank water.

## Features

- Track tank size optionally
- Time-based history
- Tag support for categorization

## Install

```bash
npm install -g @i-rs/i-rs-aqua
# or
brew install i-rs/homebrew-tap/i-rs-aqua
```

## Quick Start

```bash
# Record water change
i-rs-aqua add
i-rs-aqua add --tank-size 100

# List all records
i-rs-aqua list

# Get record details
i-rs-aqua get abc12345
```

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/aqua.json`
- Linux: `~/.config/i-rs/aqua.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0