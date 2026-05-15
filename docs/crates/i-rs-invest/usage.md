# i-rs-invest Usage Guide

## Global Flags

- `--json` — Output in JSON format

## Commands Overview

| Command | Description |
|---------|-------------|
| `add` | Add a new investment |
| `list` | List all investments with optional filters |
| `get` | Get detailed information about an investment |
| `update` | Update investment details |
| `delete` | Delete an investment |
| `stats` | View portfolio statistics |
| `example` | Show usage examples |
| `skill` | View AI skill documentation |

## add

Add a new investment to your portfolio.

```bash
i-rs-invest add <NAME> [OPTIONS]
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `NAME` | Investment name (e.g., "Apple") | Yes |

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-s` | `--symbol` | Ticker symbol (e.g., "AAPL") | Yes |
| `-t` | `--type` | Asset type (stock, fund, crypto) | Yes |
| `-q` | `--quantity` | Number of units | Yes |
| `-p` | `--price` | Buy price per unit | Yes |
| `-d` | `--date` | Buy date (YYYY-MM-DD) | No |
| | `--current-price` | Current price per unit | No |
| | `--tag` | Tags (can be specified multiple times) | No |
| | `--remark` | Remarks (can be specified multiple times) | No |

### Examples

```bash
# Add a stock
i-rs-invest add Apple --symbol AAPL --type stock --quantity 10 --price 150.00

# Add a fund with date
i-rs-invest add "S&P 500 Fund" --symbol VOO --type fund --quantity 100 --price 400 --date 2024-01-15

# Add a cryptocurrency
i-rs-invest add Bitcoin --symbol BTC --type crypto --quantity 0.5 --price 40000

# Add with tags and remarks
i-rs-invest add Tesla --symbol TSLA --type stock --qty 15 --price 250 --tag tech --tag growth --remark "Long term hold"
```

---

## list

List all investments with optional filtering.

```bash
i-rs-invest list [OPTIONS]
```

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-t` | `--type` | Filter by asset type (stock, fund, crypto) |
| | `--tag` | Filter by tag |

### Examples

```bash
# List all investments
i-rs-invest list

# List only stocks
i-rs-invest list --type stock

# List only funds
i-rs-invest list --type fund

# List only cryptocurrencies
i-rs-invest list --type crypto

# Filter by tag
i-rs-invest list --tag tech
```

---

## get

Get detailed information about a specific investment.

```bash
i-rs-invest get <NAME>
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `NAME` | Investment name | Yes |

### Examples

```bash
# Get details
i-rs-invest get Apple

# JSON output
i-rs-invest get Apple --json
```

---

## update

Update investment details.

```bash
i-rs-invest update <NAME> [OPTIONS]
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `NAME` | Investment name | Yes |

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-s` | `--symbol` | New ticker symbol |
| `-t` | `--type` | New asset type |
| `-q` | `--quantity` | New quantity |
| `-p` | `--price` | New buy price |
| | `--current-price` | New current price |
| | `--tag` | New tags (replaces all) |
| | `--remark` | New remarks (replaces all) |

### Examples

```bash
# Update current price
i-rs-invest update Apple --current-price 175.50

# Update multiple fields
i-rs-invest update Apple --current-price 180 --tag tech --remark "Strong performance"

# Update quantity
i-rs-invest update Tesla --quantity 20
```

---

## delete

Delete an investment from your portfolio.

```bash
i-rs-invest delete <NAME>
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `NAME` | Investment name | Yes |

### Examples

```bash
# Delete an investment
i-rs-invest delete Apple
```

---

## stats

View comprehensive portfolio statistics.

```bash
i-rs-invest stats
```

### Output Includes

- **Total Cost**: Sum of all investment costs
- **Total Value**: Current portfolio value
- **Total Profit/Loss**: Absolute and percentage gain/loss
- **By Asset Type**: Breakdown for stocks, funds, and cryptocurrencies
- **Individual Performance**: Top performers sorted by P/L percentage

---

## example

Show usage examples for all commands.

```bash
i-rs-invest example
```

---

## skill

View AI skill documentation.

```bash
i-rs-invest skill [SUB_COMMAND]
```

### Sub Commands

| Command | Description |
|---------|-------------|
| `summary` | Show skill summary |
| `content` | Show full skill documentation |
| `raw` | Show raw skill document |

### Examples

```bash
# Show skill documentation
i-rs-invest skill

# Show summary only
i-rs-invest skill summary
```

### data

Manage data (export, import, clear).

```bash
i-rs-invest data export
i-rs-invest data import [FILE]
i-rs-invest data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
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

## Data Storage

- macOS: `~/.config/i-rs/invest.json`
- Linux: `~/.config/i-rs/invest.json`
- Windows: `~\AppData\Roaming\i-rs\invest.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-invest list
```
