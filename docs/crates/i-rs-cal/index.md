# i-rs-cal

Calorie tracking CLI tool for estimating and recording calorie intake.

## Overview

i-rs-cal helps you track calorie intake. Record food items with calorie estimates, view daily totals, and maintain healthy eating habits.

## Quick Start

```bash
# Record calorie intake
i-rs-cal add "Apple" 95
i-rs-cal add "Pizza" 285 --tag lunch
i-rs-cal add "Burger" 350 --date 2024-01-15

# List all records
i-rs-cal list

# Get record details
i-rs-cal get abc12345

# Delete a record
i-rs-cal delete abc12345
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-cal

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-cal
```

## Data Storage

- macOS: `~/.config/i-rs/cal.json`
- Linux: `~/.config/i-rs/cal.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Features

- **Quick Entry**: Record food and calories
- **Tag Support**: Categorize by meal, food type, etc.
- **Date Tracking**: Track intake by date

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records