# i-rs-sheet

Sheet change tracking CLI tool for recording when you change bed sheets.

## Overview

i-rs-sheet helps you track when you change bed sheets. Maintain hygiene by keeping track of sheet change schedules.

## Quick Start

```bash
# Record sheet change
i-rs-sheet add bedsheet
i-rs-sheet add pillowcase --tag bedroom

# List all records
i-rs-sheet list

# Get record details
i-rs-sheet get abc12345

# Delete a record
i-rs-sheet delete abc12345
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-sheet

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-sheet
```

## Data Storage

- macOS: `~/.config/i-rs/sheets.json`
- Linux: `~/.config/i-rs/sheets.json`
- Windows: `~\AppData\Roaming\i-rs\sheet.json`

## Sheet Types

- `bedsheet` - Bottom or top sheet
- `pillowcase` - Pillow cover
- `duvet-cover` - Comforter/duvet cover
- `mattress-protector` - Mattress protector

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records