# i-rs-walkdog

Dog walking tracking CLI tool for recording when you walk your dog.

## Overview

i-rs-walkdog helps you track dog walking schedules. Record walk durations, times, and maintain exercise routines.

## Quick Start

```bash
# Record dog walk
i-rs-walkdog add "Buddy" 30
i-rs-walkdog add "Max" 45 --tag morning

# List all records
i-rs-walkdog list

# Get record details
i-rs-walkdog get abc12345

# Delete a record
i-rs-walkdog delete abc12345
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-walkdog

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-walkdog
```

## Data Storage

- macOS: `~/.config/i-rs/walkdog.json`
- Linux: `~/.config/i-rs/walkdog.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Features

- **Duration Tracking**: Record walk duration in minutes
- **Multi-dog Support**: Track multiple dogs
- **Tag Support**: Categorize walks

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records