# i-rs-feedpet

Pet feeding tracking CLI tool for recording when you feed your pets.

## Overview

i-rs-feedpet helps you track pet feeding schedules. Record pet names, food types, and amounts to maintain feeding schedules.

## Quick Start

```bash
# Record pet feeding
i-rs-feedpet add "Cat" dry-food "50g"
i-rs-feedpet add "Dog" wet-food "200g" --tag morning

# List all records
i-rs-feedpet list

# Get record details
i-rs-feedpet get abc12345

# Delete a record
i-rs-feedpet delete abc12345
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-feedpet

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-feedpet
```

## Data Storage

- macOS: `~/.config/i-rs/feedpet.json`
- Linux: `~/.config/i-rs/feedpet.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Features

- **Multi-pet Support**: Track multiple pets
- **Food Types**: Record different food types
- **Amount Tracking**: Track feeding amounts

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records