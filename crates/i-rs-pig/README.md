# i-rs-pig

Craving and junk food tracking CLI tool for recording food cravings and indulgences.

## Features

- Record craving moments and junk food
- Track food names and descriptions
- Time-based history
- Tag support for categorization

## Install

```bash
npm install -g @i-rs/i-rs-pig
# or
brew install i-rs/homebrew-tap/i-rs-pig
```

## Quick Start

```bash
# Record a craving
i-rs-pig add "Chocolate bar"
i-rs-pig add "French fries" --remark "Fast food lunch"

# List all records
i-rs-pig list

# Get record details
i-rs-pig get abc12345
```

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/pig.json`
- Linux: `~/.config/i-rs/pig.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0