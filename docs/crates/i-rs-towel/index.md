# i-rs-towel

Towel replacement tracking CLI tool for recording when you replace towels.

## Overview

i-rs-towel helps you track towel replacement schedules. Maintain hygiene by replacing towels regularly.

## Quick Start

```bash
# Record towel replacement
i-rs-towel add bath
i-rs-towel add face --tag bedroom

# List all records
i-rs-towel list

# Get record details
i-rs-towel get abc12345

# Delete a record
i-rs-towel delete abc12345
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-towel

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-towel
```

## Data Storage

- macOS: `~/.config/i-rs/towels.json`
- Linux: `~/.config/i-rs/towels.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Towel Types

- `bath` - Bath towel
- `face` - Face towel
- `hand` - Hand towel
- `beach` - Beach towel
- `sports` - Sports towel

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records