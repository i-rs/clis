---
name: "i-rs-invest"
description: "Tracks investment returns for stocks, funds, and cryptocurrencies. Invoked when user needs to add/view/update/delete investments, calculate profit/loss, or view portfolio statistics."
---

# i-rs-invest

Investment returns tracking CLI tool for stocks, funds, and cryptocurrencies.

## Storage

- Config: `~/.config/i-rs/invest.json`

## Global Flags

- `--json` — Output in JSON format
## Commands

### add

Add a new investment.

```bash
i-rs-invest add <NAME> [OPTIONS]
```

Options:
- `-s, --symbol <SYMBOL>` - Ticker symbol (e.g., AAPL, VOO, BTC)
- `-t, --type <TYPE>` - Asset type: stock, fund, or crypto
- `-q, --quantity <QUANTITY>` - Number of units held
- `-p, --price <PRICE>` - Buy price per unit
- `-d, --date <DATE>` - Buy date (YYYY-MM-DD)
- `--current-price <PRICE>` - Current price per unit
- `--tag <TAG>` - Tags (can be repeated)
- `--remark <REMARK>` - Remarks (can be repeated)

### list

List all investments.

```bash
i-rs-invest list [OPTIONS]
```

Options:
- `-t, --type <TYPE>` - Filter by asset type (stock, fund, crypto)
- `--tag <TAG>` - Filter by tag

### get

Get detailed information about an investment.

```bash
i-rs-invest get <NAME>
```

### update

Update investment details.

```bash
i-rs-invest update <NAME>
```

Options:
- `-s, --symbol <SYMBOL>` - New ticker symbol
- `-t, --type <TYPE>` - New asset type
- `-q, --quantity <QUANTITY>` - New quantity
- `-p, --price <PRICE>` - New buy price
- `--current-price <PRICE>` - New current price
- `--tag <TAG>` - New tags
- `--remark <REMARK>` - New remarks

### delete

Delete an investment.

```bash
i-rs-invest delete <NAME>
```

### stats

View portfolio statistics.

```bash
i-rs-invest stats
```

Shows total cost, value, profit/loss, and breakdown by asset type.

### data

Manage data (export, import, clear).

```bash
i-rs-invest data export
i-rs-invest data import [FILE]
i-rs-invest data clear
```

### example

Show usage examples.

```bash
i-rs-invest example
```

### skill

Show skill information.

```bash
i-rs-invest skill [summary|content|raw]
```

## Examples

```bash
# Add a stock
i-rs-invest add Apple --symbol AAPL --type stock --quantity 10 --price 150.00 [OPTIONS]

# Add a fund
i-rs-invest add "S&P 500 Fund" --symbol VOO --type fund --qty 50 --price 400 [OPTIONS]

# Add a cryptocurrency
i-rs-invest add Bitcoin --symbol BTC --type crypto --quantity 0.5 --price 40000 [OPTIONS]

# Update current price
i-rs-invest update Apple --current-price 175.50

# List all investments
i-rs-invest list [OPTIONS]

# Filter by type
i-rs-invest list --type stock

# View statistics
i-rs-invest stats

# Get detailed information
i-rs-invest get Apple

# Delete an investment
i-rs-invest delete Apple
```
