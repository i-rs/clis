# i-rs-cycling

Cycling record tracking CLI tool for recording and managing cycling activities.

## Features

- Record cycling activities with date, distance, and duration
- Track elevation gain for hill climbing
- Add route descriptions and custom tags
- Automatic average speed calculation
- View cumulative statistics
- Tag-based filtering
- JSON output support

## Install

```bash
npm install -g @i-rs/i-rs-cycling
# or
brew install i-rs/homebrew-tap/i-rs-cycling
```

## Quick Start

```bash
# Add a cycling record
i-rs-cycling add 2025-06-14 25.5 60 --elevation 300

# List all records
i-rs-cycling list

# View statistics
i-rs-cycling stats

# Get record details
i-rs-cycling get <uuid>
```

## Data Storage

- macOS: `~/.config/i-rs/cycling.json`
- Linux: `~/.config/i-rs/cycling.json`
- Windows: `~\AppData\Roaming\i-rs\cycling.json`

## License

MIT OR Apache-2.0
