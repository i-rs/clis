# i-rs-water

Water intake tracking CLI tool for recording how much water you drink daily.

## Features

- Quick water intake recording in milliliters
- Daily intake tracking
- Tag support for categorization
- Time-based history

## Install

```bash
npm install -g @i-rs/i-rs-water
# or
brew install i-rs/homebrew-tap/i-rs-water
```

## Quick Start

```bash
# Record water intake
i-rs-water add 250
i-rs-water add 500 --tag morning

# List all records
i-rs-water list

# Get record details
i-rs-water get abc12345
```

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/water.json`
- Linux: `~/.config/i-rs/water.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0