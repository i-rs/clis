# i-rs-kv

Key-value storage CLI tool for storing and retrieving arbitrary data.

## Overview

i-rs-kv provides a simple key-value storage solution for storing configuration, notes, and other data you need to access quickly.

## Quick Start

```bash
# Set a value
i-rs-kv add "username" --value "john"

# Get a value
i-rs-kv get username

# List all entries
i-rs-kv list
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-kv

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-kv
```

## Data Storage

- macOS: `~/.config/i-rs/kv.json`
- Linux: `~/.config/i-rs/kv.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Features

- **Simple Storage**: Key-value pairs
- **Tag Support**: Categorize entries with tags
- **Time Metadata**: Track creation and updates
- **Quick Access**: Fast get/set operations

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records