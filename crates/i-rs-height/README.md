# i-rs-height

Height tracking CLI tool for monitoring body height and weight over time.

## Features

- Record height measurements with optional weight
- View history with ASCII charts
- Calculate min/max/average statistics
- Track height change over time
- Set target height goals
- Tag and remark support
- JSON output support

## Install

```bash
npm install -g @i-rs/i-rs-height
# or
brew install i-rs/homebrew-tap/i-rs-height
```

## Quick Start

```bash
# Add height record
i-rs-height add 2025-06-14 175.5

# Add with weight
i-rs-height add 2025-06-14 175.5 --weight 68.5

# List all records
i-rs-height list

# List with chart
i-rs-height list --chart

# List with stats
i-rs-height list --stats

# Get specific record
i-rs-height get 2025-06-14

# Delete record
i-rs-height delete 2025-06-15

# Set target height
i-rs-height set 180.0

# Show target
i-rs-height target
```

## Data Storage

- macOS: `~/.config/i-rs/heights.json`
- Linux: `~/.config/i-rs/heights.json`
- Windows: `~\AppData\Roaming\i-rs\heights.json`

## License

MIT OR Apache-2.0
