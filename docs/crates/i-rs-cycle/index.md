# i-rs-cycle

Menstrual cycle tracking CLI tool for recording period, spotting, and symptoms.

## Overview

i-rs-cycle helps you track your menstrual cycle. Record different event types, symptoms, and view your cycle history.

## Quick Start

```bash
# Record cycle event
i-rs-cycle add 2024-01-15 period
i-rs-cycle add 2024-01-20 spotting --symptom cramps

# List all records
i-rs-cycle list

# Get record details
i-rs-cycle get abc12345

# Delete a record
i-rs-cycle delete abc12345
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-cycle

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-cycle
```

## Data Storage

- macOS: `~/.config/i-rs/cycles.json`
- Linux: `~/.config/i-rs/cycles.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Event Types

- `period` - Menstrual period
- `spotting` - Light bleeding
- `ovulation` - Ovulation day
- `fertile` - Fertile window

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records