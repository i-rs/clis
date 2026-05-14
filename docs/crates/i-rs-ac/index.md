# i-rs-ac

AC cleaning tracking CLI tool for recording when you clean air conditioners.

## Overview

i-rs-ac helps you track air conditioner cleaning schedules. Maintain healthy air quality by keeping track of cleaning schedules.

## Quick Start

```bash
# Record AC cleaning
i-rs-ac add "Living Room"
i-rs-ac add "Bedroom" --tag summer

# List all records
i-rs-ac list

# Get record details
i-rs-ac get abc12345

# Delete a record
i-rs-ac delete abc12345
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-ac

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-ac
```

## Data Storage

- macOS: `~/.config/i-rs/ac.json`
- Linux: `~/.config/i-rs/ac.json`
- Windows: `~\AppData\Roaming\i-rs\ac.json`

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records