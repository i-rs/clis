# i-rs-want

Wish list CLI tool for tracking things you want to buy or get.

## Overview

i-rs-want helps you manage your wish list. Track items you want, their prices, and priority.

## Quick Start

```bash
# Add wish list items
i-rs-want add "New Headphones" --price 299.99 --priority high
i-rs-want add "Book: Rust Programming" --price 49.99 --priority medium

# List all items
i-rs-want list
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-want

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-want
```

## Data Storage

- macOS: `~/.config/i-rs/want.json`
- Linux: `~/.config/i-rs/want.json`
- Windows: `~\AppData\Roaming\i-rs\want.json`

## Features

- **Item Tracking**: Track wanted items
- **Price Tracking**: Optional prices
- **Priority Levels**: low, medium, high
- **Done/Pending**: Mark items as acquired

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records