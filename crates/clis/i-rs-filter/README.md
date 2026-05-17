# i-rs-filter

Appliance filter cleaning tracking CLI tool for recording when you clean appliance filters.

## Features

- Track different appliances and filter types
- Time-based history
- Tag support for categorization

## Install

```bash
npm install -g @i-rs/i-rs-filter
# or
brew install i-rs/homebrew-tap/i-rs-filter
```

## Quick Start

```bash
# Record filter cleaning
i-rs-filter add "Air Purifier" HEPA
i-rs-filter add "Vacuum" dust --tag living-room

# List all records
i-rs-filter list

# Get record details
i-rs-filter get abc12345
```

## Common Filter Types

- `HEPA` - High-efficiency particulate air
- `Carbon` - Activated carbon filter
- `Foam` - Foam filter
- `Dust` - Dust collection filter

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/filters.json`
- Linux: `~/.config/i-rs/filters.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0