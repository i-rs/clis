# i-rs-bestby

Item best-by date tracking CLI tool for recording purchase dates and replacement cycles.

## Overview

i-rs-bestby helps you track when items were purchased and when they need to be replaced. Perfect for tracking perishable items, consumables, and equipment with limited lifespans.

## Quick Start

```bash
# Add item with purchase date and cycle
i-rs-bestby add "Milk" --purchase-date 2024-01-01 --cycle-days 7

# List all items
i-rs-bestby list

# Get item details
i-rs-bestby get Milk
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-bestby

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-bestby
```

## Data Storage

- macOS: `~/.config/i-rs/bestby.json`
- Linux: `~/.config/i-rs/bestby.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Features

- **Purchase Tracking**: Record when items were purchased
- **Cycle Management**: Set replacement cycles in days
- **Expiration Status**: Shows OK, SOON, or EXPIRED status
- **Tag Support**: Categorize items with tags

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records