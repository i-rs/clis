# i-rs-sit

Sitting duration tracking CLI tool for recording how long you've been sitting.

## Overview

i-rs-sit helps you track sitting duration to maintain healthy habits. Record how long you've been sitting, get reminders to move.

## Quick Start

```bash
# Record sitting duration
i-rs-sit add 60
i-rs-sit add 120 --tag work

# List all records
i-rs-sit list

# Get record details
i-rs-sit get abc12345

# Delete a record
i-rs-sit delete abc12345
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-sit

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-sit
```

## Data Storage

- macOS: `~/.config/i-rs/sit.json`
- Linux: `~/.config/i-rs/sit.json`
- Windows: `~\AppData\Roaming\i-rs\sit.json`

## Features

- **Duration Tracking**: Record sitting time in minutes
- **Auto Timing**: Automatically calculates start/end times
- **Tag Support**: Categorize by work, home, etc.

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records