# i-rs-aqua

Aquarium water change tracking CLI tool for recording when you change fish tank water.

## Overview

i-rs-aqua helps you track aquarium maintenance. Record water changes to maintain healthy fish tank conditions.

## Quick Start

```bash
# Record water change
i-rs-aqua add
i-rs-aqua add --tank-size 100

# List all records
i-rs-aqua list

# Get record details
i-rs-aqua get abc12345

# Delete a record
i-rs-aqua delete abc12345
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-aqua

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-aqua
```

## Data Storage

- macOS: `~/.config/i-rs/aqua.json`
- Linux: `~/.config/i-rs/aqua.json`
- Windows: `~\AppData\Roaming\i-rs\aqua.json`

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records