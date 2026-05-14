# i-rs-bed

Bed item replacement tracking CLI tool for recording when you replace mattress, pillows, etc.

## Overview

i-rs-bed helps you track bed item replacement schedules. Maintain sleep quality by replacing mattress and pillows regularly.

## Quick Start

```bash
# Record bed item replacement
i-rs-bed add mattress
i-rs-bed add pillow --tag bedroom

# List all records
i-rs-bed list

# Get record details
i-rs-bed get abc12345

# Delete a record
i-rs-bed delete abc12345
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-bed

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-bed
```

## Data Storage

- macOS: `~/.config/i-rs/beds.json`
- Linux: `~/.config/i-rs/beds.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Bed Item Types

- `mattress` - Mattress
- `pillow` - Pillow
- `duvet` - Duvet/comforter
- `mattress-protector` - Mattress protector

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records