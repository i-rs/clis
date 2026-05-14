# i-rs-water

Water intake tracking CLI tool for recording how much water you drink daily.

## Overview

i-rs-water helps you track your daily water intake. Record how much water you drink, view history, and maintain healthy hydration habits.

## Quick Start

```bash
# Record water intake
i-rs-water add 250
i-rs-water add 500 --tag morning

# List all records
i-rs-water list

# Get record details
i-rs-water get abc12345

# Delete a record
i-rs-water delete abc12345
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-water

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-water
```

## Data Storage

- macOS: `~/.config/i-rs/water.json`
- Linux: `~/.config/i-rs/water.json`
- Windows: `~\AppData\Roaming\i-rs\water.json`

## Features

- **Quick Recording**: Record water intake in milliliters
- **Tag Support**: Categorize entries with tags
- **Time Tracking**: Automatic timestamp for each entry
- **History View**: List all past records

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records