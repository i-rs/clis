# i-rs-invest

Investment returns tracking CLI tool for stocks, funds, and cryptocurrencies.

## Overview

i-rs-invest helps you track your investment portfolio with support for stocks, mutual funds, ETFs, and cryptocurrencies. Calculate profit/loss, view statistics, and organize investments with tags.

## Quick Start

```bash
# Add a stock investment
i-rs-invest add Apple --symbol AAPL --type stock --quantity 10 --price 150.00

# Add a fund
i-rs-invest add "S&P 500 Fund" --symbol VOO --type fund --qty 100 --price 400

# Add a cryptocurrency
i-rs-invest add Bitcoin --symbol BTC --type crypto --quantity 0.5 --price 40000

# Update current price
i-rs-invest update Apple --current-price 175.50

# List all investments
i-rs-invest list

# Filter by type
i-rs-invest list --type stock

# View statistics
i-rs-invest stats

# Get detailed information
i-rs-invest get Apple
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-invest

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-invest
```

## Data Storage

- macOS: `~/.config/i-rs/invest.json`
- Linux: `~/.config/i-rs/invest.json`
- Windows: `~\AppData\Roaming\i-rs\invest.json`

## Features

- **Multi-Asset Support**: Track stocks, funds, and cryptocurrencies
- **Profit/Loss Calculation**: Automatic P/L percentage calculation
- **Portfolio Statistics**: Overview of total cost, value, and performance
- **Tag System**: Organize investments with custom tags
- **Flexible Filtering**: Filter by asset type or tags

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records
