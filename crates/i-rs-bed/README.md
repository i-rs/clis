# i-rs-bed

Bed item replacement tracking CLI tool for recording when you replace mattress, pillows, etc.

## Features

- Track bed item types (mattress, pillow, etc.)
- Time-based history
- Tag support for categorization

## Install

```bash
npm install -g @i-rs/i-rs-bed
# or
brew install i-rs/homebrew-tap/i-rs-bed
```

## Quick Start

```bash
# Record bed item replacement
i-rs-bed add mattress
i-rs-bed add pillow --tag bedroom

# List all records
i-rs-bed list

# Get record details
i-rs-bed get abc12345
```

## Bed Item Types

- `mattress` - Mattress replacement
- `pillow` - Pillow replacement
- `duvet` - Duvet/comforter
- `mattress-protector` - Mattress protector

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/beds.json`
- Linux: `~/.config/i-rs/beds.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0