# i-rs-purify

Water purifier filter replacement tracking CLI tool.

## Overview

i-rs-purify helps you track water purifier filter replacement schedules. Maintain clean drinking water.

## Quick Start

```bash
# Record filter replacement
i-rs-purify add "RO Membrane"
i-rs-purify add "Carbon Filter" --tag kitchen

# List all records
i-rs-purify list

# Get record details
i-rs-purify get abc12345

# Delete a record
i-rs-purify delete abc12345
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-purify

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-purify
```

## Data Storage

- macOS: `~/.config/i-rs/purify.json`
- Linux: `~/.config/i-rs/purify.json`
- Windows: `~\AppData\Roaming\i-rs\purify.json`

## Filter Types

- `RO Membrane` - Reverse osmosis membrane
- `Carbon Filter` - Activated carbon filter
- `Sediment Filter` - Pre-filter for sediment
- `Mineral Filter` - Post-filter adding minerals

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records