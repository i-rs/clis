# i-rs-pig

Craving and junk food tracking CLI tool for recording food cravings and indulgences.

## Overview

i-rs-pig helps you track moments of unhealthy eating or cravings. Sometimes we give in - it's good to be aware of it.

## Quick Start

```bash
# Record a craving
i-rs-pig add "Chocolate bar"
i-rs-pig add "French fries" --remark "Fast food lunch"

# List all records
i-rs-pig list
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-pig

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-pig
```

## Data Storage

- macOS: `~/.config/i-rs/pig.json`
- Linux: `~/.config/i-rs/pig.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Features

- **Simple Tracking**: Record cravings and indulgences
- **Description**: Add details about the food
- **Time History**: Automatic timestamp
- **Tag Support**: Categorize entries

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records