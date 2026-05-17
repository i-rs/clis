# i-rs-purify

Water purifier filter replacement tracking CLI tool.

## Features

- Track filter types
- Time-based history
- Tag support for categorization

## Install

```bash
npm install -g @i-rs/i-rs-purify
# or
brew install i-rs/homebrew-tap/i-rs-purify
```

## Quick Start

```bash
# Record filter replacement
i-rs-purify add "RO Membrane"
i-rs-purify add "Carbon Filter" --tag kitchen

# List all records
i-rs-purify list

# Get record details
i-rs-purify get abc12345
```

## Common Filter Types

- `RO Membrane` - Reverse osmosis membrane
- `Carbon Filter` - Activated carbon filter
- `Sediment Filter` - Pre-filter for sediment
- `Mineral Filter` - Post-filter adding minerals

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/purify.json`
- Linux: `~/.config/i-rs/purify.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0