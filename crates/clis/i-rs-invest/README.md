# i-rs-invest

Investment returns tracking CLI tool for stocks, funds, and cryptocurrencies.

## Features

- Track stocks, funds, and cryptocurrency holdings
- Record buy price, quantity, and date
- Calculate profit/loss percentage
- Filter by asset type or tags
- Portfolio statistics and performance analysis
- JSON output support

## Install

```bash
npm install -g @i-rs/i-rs-invest
# or
brew install i-rs/homebrew-tap/i-rs-invest
```

## Quick Start

```bash
# Add a stock investment
i-rs-invest add Apple --symbol AAPL --type stock --quantity 10 --price 150.00

# Add a fund
i-rs-invest add "S&P 500 Fund" --symbol VOO --type fund --qty 100 --price 400 --tag etf

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

## Commands

- **add** - Add a new investment
- **list** - List all investments with optional filters
- **get** - Get detailed information about an investment
- **update** - Update investment details
- **delete** - Delete an investment
- **stats** - View portfolio statistics
- **example** - Show usage examples
- **skill** - View AI skill documentation

## Data Storage

- macOS: `~/.config/i-rs/invest.json`
- Linux: `~/.config/i-rs/invest.json`
- Windows: `~\AppData\Roaming\i-rs\invest.json`

## Asset Types

- **stock** - Individual stocks (e.g., AAPL, GOOGL)
- **fund** - Mutual funds, ETFs (e.g., VTI, SPY)
- **crypto** - Cryptocurrencies (e.g., BTC, ETH)

## License

MIT OR Apache-2.0
