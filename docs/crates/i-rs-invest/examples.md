# i-rs-invest Examples

## Basic Usage

### Adding Stocks

```bash
# Add a simple stock
i-rs-invest add Apple --symbol AAPL --type stock --quantity 10 --price 150.00

# Add multiple stocks
i-rs-invest add Google --symbol GOOGL --type stock --qty 5 --price 140.00
i-rs-invest add Microsoft --symbol MSFT --type stock --qty 15 --price 380.00
i-rs-invest add Amazon --symbol AMZN --type stock --qty 8 --price 175.00

# Add with tags
i-rs-invest add Nvidia --symbol NVDA --type stock --qty 3 --price 500 --tag ai --tag semiconductor

# Add with buy date
i-rs-invest add Apple --symbol AAPL --type stock --qty 10 --price 150 --date 2024-01-15
```

### Adding Funds

```bash
# Add index funds
i-rs-invest add "S&P 500 Fund" --symbol VOO --type fund --quantity 50 --price 400.00
i-rs-invest add "Total Market Fund" --symbol VTI --type fund --qty 30 --price 220.00
i-rs-invest add "Tech Fund" --symbol QQQ --type fund --quantity 20 --price 380.00

# Add with tags
i-rs-invest add "S&P 500 Fund" --symbol VOO --type fund --qty 50 --price 400 --tag etf --tag index
i-rs-invest add "Emerging Markets" --symbol VWO --type fund --quantity 25 --price 42 --tag emerging --tag international
```

### Adding Cryptocurrencies

```bash
# Add major cryptocurrencies
i-rs-invest add Bitcoin --symbol BTC --type crypto --quantity 0.5 --price 40000.00
i-rs-invest add Ethereum --symbol ETH --type crypto --qty 2.0 --price 2500.00
i-rs-invest add Solana --symbol SOL --type crypto --quantity 20 --price 100.00

# Add with tags
i-rs-invest add Bitcoin --symbol BTC --type crypto --qty 0.5 --price 40000 --tag defi --tag store-of-value
i-rs-invest add Ethereum --symbol ETH --type crypto --qty 2.0 --price 2500 --tag defi --tag smart-contracts
```

---

## Updating Investments

### Updating Current Prices

```bash
# Update single investment
i-rs-invest update Apple --current-price 175.50
i-rs-invest update Bitcoin --current-price 45000

# Update multiple investments
i-rs-invest update Google --current-price 145.00
i-rs-invest update Microsoft --current-price 390.00
i-rs-invest update Ethereum --current-price 2800
```

### Updating Other Details

```bash
# Update quantity (e.g., after buying more)
i-rs-invest update Apple --quantity 15

# Update buy price (e.g., for adjusted cost basis)
i-rs-invest update Apple --price 155.00

# Update tags
i-rs-invest update Tesla --tag ev --tag automotive --tag tech

# Update multiple fields
i-rs-invest update Apple --current-price 180 --quantity 12 --remark "Strong quarter"
```

---

## Viewing Investments

### List All Investments

```bash
# List all investments
i-rs-invest list

# List in JSON format
i-rs-invest list --json
```

### Filter by Asset Type

```bash
# List only stocks
i-rs-invest list --type stock

# List only funds
i-rs-invest list --type fund

# List only cryptocurrencies
i-rs-invest list --type crypto
```

### Filter by Tags

```bash
# List investments with specific tag
i-rs-invest list --tag tech

# List investments with multiple tags
i-rs-invest list --tag ai
```

---

## Detailed Information

### Get Investment Details

```bash
# Get details for a specific investment
i-rs-invest get Apple

# Get in JSON format
i-rs-invest get Apple --json

# Get multiple investments
i-rs-invest get Bitcoin
i-rs-invest get Ethereum
```

---

## Portfolio Statistics

### View Statistics

```bash
# View complete portfolio stats
i-rs-invest stats

# Shows:
# - Total cost and value
# - Total profit/loss
# - Breakdown by asset type
# - Individual performance rankings
```

---

## Managing Investments

### Deleting Investments

```bash
# Delete an investment
i-rs-invest delete Apple

# Delete multiple
i-rs-invest delete Google
i-rs-invest delete Tesla
```

---

## Practical Workflows

### Weekly Price Update

```bash
#!/bin/bash
# Weekly script to update current prices

i-rs-invest update Apple --current-price 175.50
i-rs-invest update Google --current-price 145.00
i-rs-invest update Microsoft --current-price 390.00
i-rs-invest update Bitcoin --current-price 45000
i-rs-invest update Ethereum --current-price 2800

# View updated stats
i-rs-invest stats
```

### Monthly Review

```bash
#!/bin/bash
# Monthly portfolio review

echo "=== Monthly Portfolio Review ==="
echo

echo "1. Current Holdings:"
i-rs-invest list

echo
echo "2. Portfolio Statistics:"
i-rs-invest stats

echo
echo "3. Top Performers:"
i-rs-invest stats | grep -A 10 "Individual Performance"
```

### Year-End Tax Report

```bash
#!/bin/bash
# Generate tax report

echo "=== Investment Report ==="
echo "Generated: $(date)"
echo

echo "Total Portfolio:"
i-rs-invest stats

echo
echo "Stock Holdings:"
i-rs-invest list --type stock

echo
echo "Fund Holdings:"
i-rs-invest list --type fund

echo
echo "Crypto Holdings:"
i-rs-invest list --type crypto
```

---

## Advanced Usage

### Using Tags for Organization

```bash
# Add investments with multiple tags
i-rs-invest add Apple --symbol AAPL --type stock --qty 10 --price 150 \
  --tag tech --tag dividend --tag large-cap

i-rs-invest add "Small Cap Fund" --symbol VB --type fund --qty 30 --price 200 \
  --tag small-cap --tag growth --tag diversification

i-rs-invest add Bitcoin --symbol BTC --type crypto --qty 0.5 --price 40000 \
  --tag store-of-value --tag defi --tag high-risk
```

### Creating a Diversified Portfolio

```bash
#!/bin/bash
# Set up a diversified portfolio

# US Stocks (40%)
i-rs-invest add "S&P 500 ETF" --symbol VOO --type fund --qty 40 --price 400 --tag us-stocks --tag index
i-rs-invest add "Total Market ETF" --symbol VTI --type fund --qty 30 --price 220 --tag us-stocks --tag broad-market

# International Stocks (20%)
i-rs-invest add "Intl Developed" --symbol VEA --type fund --qty 25 --price 45 --tag international --tag developed
i-rs-invest add "Emerging Markets" --symbol VWO --type fund --qty 20 --price 42 --tag international --tag emerging

# Bonds (20%)
i-rs-invest add "Total Bond ETF" --symbol BND --type fund --qty 30 --price 75 --tag bonds --tag fixed-income

# Crypto (10%)
i-rs-invest add Bitcoin --symbol BTC --type crypto --qty 0.1 --price 40000 --tag crypto --tag high-risk
i-rs-invest add Ethereum --symbol ETH --type crypto --qty 1.0 --price 2500 --tag crypto --tag high-risk

# Real Assets (10%)
i-rs-invest add "Gold ETF" --symbol GLD --type fund --qty 10 --price 180 --tag commodities --tag inflation-hedge
```

### Performance Tracking

```bash
#!/bin/bash
# Track performance over time

# Day 1: Add investments
i-rs-invest add Apple --symbol AAPL --type stock --qty 10 --price 150 --date 2024-01-01
i-rs-invest add Bitcoin --symbol BTC --type crypto --qty 0.5 --price 40000 --date 2024-01-01

# Month 1: Update prices
i-rs-invest update Apple --current-price 165
i-rs-invest update Bitcoin --current-price 42000
i-rs-invest stats

# Month 2: Update prices
i-rs-invest update Apple --current-price 175
i-rs-invest update Bitcoin --current-price 45000
i-rs-invest stats
```
