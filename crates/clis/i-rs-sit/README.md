# i-rs-sit

Sitting duration tracking CLI tool for recording how long you've been sitting.

## Features

- Track sitting duration in minutes
- Auto-calculate start/end times
- Tag support for categorization
- Time-based history

## Install

```bash
npm install -g @i-rs/i-rs-sit
# or
brew install i-rs/homebrew-tap/i-rs-sit
```

## Quick Start

```bash
# Record sitting duration
i-rs-sit add 60
i-rs-sit add 120 --tag work

# List all records
i-rs-sit list

# Get record details
i-rs-sit get abc12345
```

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/sit.json`
- Linux: `~/.config/i-rs/sit.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0