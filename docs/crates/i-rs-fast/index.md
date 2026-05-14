# i-rs-fast

Fasting tracking CLI tool for recording fasting sessions.

## Overview

i-rs-fast helps you track fasting sessions. Record start times, target hours, and monitor your fasting progress.

## Quick Start

```bash
# Start fasting
i-rs-fast add 16
i-rs-fast add 24 --tag omad

# List all records
i-rs-fast list

# Get record details
i-rs-fast get abc12345

# Delete a record
i-rs-fast delete abc12345
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-fast

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-fast
```

## Data Storage

- macOS: `~/.config/i-rs/fast.json`
- Linux: `~/.config/i-rs/fast.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Features

- **Target Tracking**: Set target fasting hours
- **Auto Timing**: Records start time automatically
- **Tag Support**: Categorize fasting types

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records