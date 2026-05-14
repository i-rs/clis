# i-rs-recur

Recurring expenses tracking CLI tool for managing fixed expenses like rent and subscriptions.

## Overview

i-rs-recur helps you track recurring expenses and know when the next payment is due.

## Quick Start

```bash
# Add recurring expense
i-rs-recur add "Netflix" --amount 15.99 --frequency monthly
i-rs-recur add "Rent" --amount 2000 --frequency monthly

# List all
i-rs-recur list
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-recur

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-recur
```

## Data Storage

- macOS: `~/.config/i-rs/recur.json`
- Linux: `~/.config/i-rs/recur.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Features

- **Multiple Frequencies**: daily, weekly, monthly, quarterly, yearly
- **Next Payment Calculation**: Automatic countdown
- **Tag Support**: Categorize expenses
- **Amount Tracking**: Track cost over time

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records