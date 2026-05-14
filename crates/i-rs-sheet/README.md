# i-rs-sheet

Sheet change tracking CLI tool for recording when you change bed sheets.

## Features

- Track different sheet types (bedsheet, pillowcase, etc.)
- Time-based history
- Tag support for categorization

## Install

```bash
npm install -g @i-rs/i-rs-sheet
# or
brew install i-rs/homebrew-tap/i-rs-sheet
```

## Quick Start

```bash
# Record sheet change
i-rs-sheet add bedsheet
i-rs-sheet add pillowcase --tag bedroom

# List all records
i-rs-sheet list

# Get record details
i-rs-sheet get abc12345
```

## Sheet Types

- `bedsheet` - Bottom or top sheet
- `pillowcase` - Pillow cover
- `duvet-cover` - Comforter/duvet cover
- `mattress-protector` - Mattress protector

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/sheets.json`
- Linux: `~/.config/i-rs/sheets.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0