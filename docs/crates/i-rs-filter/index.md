# i-rs-filter

Appliance filter cleaning tracking CLI tool for recording when you clean appliance filters.

## Overview

i-rs-filter helps you track appliance filter cleaning schedules. Maintain air quality and appliance efficiency.

## Quick Start

```bash
# Record filter cleaning
i-rs-filter add "Air Purifier" HEPA
i-rs-filter add "Vacuum" dust --tag living-room

# List all records
i-rs-filter list

# Get record details
i-rs-filter get abc12345

# Delete a record
i-rs-filter delete abc12345
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-filter

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-filter
```

## Data Storage

- macOS: `~/.config/i-rs/filters.json`
- Linux: `~/.config/i-rs/filters.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Filter Types

- `HEPA` - High-efficiency particulate air
- `Carbon` - Activated carbon filter
- `Foam` - Foam filter
- `Dust` - Dust collection filter

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records