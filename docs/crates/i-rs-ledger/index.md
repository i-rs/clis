# i-rs-ledger

Personal accounting CLI tool for tracking income, expenses, and transfers.

## Overview

i-rs-ledger helps you manage your personal finances by tracking income, expenses, and transfers with category-based organization.

## Quick Start

```bash
# Add income
i-rs-ledger add --type income --amount 5000 --category salary

# Add expense
i-rs-ledger add --type expense --amount 150 --category food

# List entries
i-rs-ledger list
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-ledger

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-ledger
```

## Data Storage

- macOS: `~/.config/i-rs/ledger.json`
- Linux: `~/.config/i-rs/ledger.json`
- Windows: `~\AppData\Roaming\i-rs\ledger.json`

## Features

- **Transaction Types**: Income, expense, and transfer
- **Categories**: Flexible category system
- **Multi-currency**: Support for different currencies
- **Time Tracking**: Automatic timestamps

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records