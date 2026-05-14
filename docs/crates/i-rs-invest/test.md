# i-rs-invest Test Records

Use these test records to verify your installation and explore functionality.

## Test Setup

### 1. Add Test Investments

```bash
# Add stocks
i-rs-invest add Apple --symbol AAPL --type stock --quantity 10 --price 150.00 --date 2024-01-15
i-rs-invest add Google --symbol GOOGL --type stock --quantity 5 --price 140.00 --date 2024-02-01
i-rs-invest add Microsoft --symbol MSFT --type stock --quantity 15 --price 380.00 --date 2024-03-01

# Add funds
i-rs-invest add "S&P 500 ETF" --symbol VOO --type fund --quantity 50 --price 400.00 --date 2024-01-10
i-rs-invest add "Total Market ETF" --symbol VTI --type fund --quantity 30 --price 220.00 --date 2024-02-15

# Add cryptocurrency
i-rs-invest add Bitcoin --symbol BTC --type crypto --quantity 0.5 --price 40000.00 --date 2024-01-20
i-rs-invest add Ethereum --symbol ETH --type crypto --quantity 2.0 --price 2500.00 --date 2024-03-05
```

### 2. Update Current Prices

```bash
# Update stocks
i-rs-invest update Apple --current-price 175.50
i-rs-invest update Google --current-price 145.00
i-rs-invest update Microsoft --current-price 390.00

# Update funds
i-rs-invest update "S&P 500 ETF" --current-price 420.00
i-rs-invest update "Total Market ETF" --current-price 235.00

# Update crypto
i-rs-invest update Bitcoin --current-price 45000.00
i-rs-invest update Ethereum --current-price 2800.00
```

### 3. View Results

#### List All Investments

```bash
i-rs-invest list
```

Expected output should show all 7 investments with:
- Name, Symbol, Type
- Quantity and Buy Price
- Current Price
- Profit/Loss Percentage
- Current Value

#### View Statistics

```bash
i-rs-invest stats
```

Expected output should include:
- Total Cost
- Total Value
- Total Profit/Loss
- Breakdown by type (Stocks, Funds, Crypto)
- Individual performance rankings

#### Get Specific Investment

```bash
i-rs-invest get Apple
```

Expected output should show detailed information:
- All investment details
- Total cost
- Current value
- Profit/Loss

## Filter Tests

### Filter by Type

```bash
# Test stock filter
i-rs-invest list --type stock

# Test fund filter
i-rs-invest list --type fund

# Test crypto filter
i-rs-invest list --type crypto
```

### Filter by Tag

```bash
# Add with tags first
i-rs-invest add Tesla --symbol TSLA --type stock --qty 8 --price 250 --tag tech --tag ev

# Filter by tag
i-rs-invest list --tag tech
i-rs-invest list --tag ev
```

## Update Tests

### Update Current Price

```bash
# Update multiple times
i-rs-invest update Apple --current-price 180.00
i-rs-invest update Apple --current-price 185.00
i-rs-invest update Apple --current-price 190.00

# Check if update works
i-rs-invest get Apple
```

### Update Multiple Fields

```bash
# Update multiple fields at once
i-rs-invest update Apple --current-price 195.00 --quantity 12

# Verify updates
i-rs-invest get Apple
```

## Delete Tests

### Delete Investment

```bash
# Add test investment
i-rs-invest add "Test Stock" --symbol TEST --type stock --qty 100 --price 10.00

# Delete it
i-rs-invest delete "Test Stock"

# Verify deletion
i-rs-invest list
i-rs-invest get "Test Stock"  # Should show error
```

## JSON Output Tests

```bash
# List in JSON
i-rs-invest list --json

# Get in JSON
i-rs-invest get Apple --json

# Check JSON structure
i-rs-invest list --json | jq '.'
```

## Error Handling Tests

### Invalid Input

```bash
# Test with negative quantity
i-rs-invest add "Bad Stock" --symbol BAD --type stock --quantity -10 --price 100.00

# Test with zero price
i-rs-invest add "Bad Stock" --symbol BAD --type stock --quantity 10 --price 0.00

# Test with invalid type
i-rs-invest add "Test" --symbol T --type invalid --quantity 10 --price 100.00
```

### Non-existent Investment

```bash
# Try to get non-existent
i-rs-invest get "Non Existent"

# Try to delete non-existent
i-rs-invest delete "Non Existent"

# Try to update non-existent
i-rs-invest update "Non Existent" --current-price 100.00
```

## Cleanup

```bash
# Delete all test investments
i-rs-invest delete Apple
i-rs-invest delete Google
i-rs-invest delete Microsoft
i-rs-invest delete "S&P 500 ETF"
i-rs-invest delete "Total Market ETF"
i-rs-invest delete Bitcoin
i-rs-invest delete Ethereum
i-rs-invest delete Tesla
i-rs-invest delete "Test Stock"

# Verify cleanup
i-rs-invest list
```

## Expected Results After Test

After running all tests successfully:

1. **Add**: All test investments should be created successfully
2. **List**: Should display all investments in a formatted table
3. **Stats**: Should show correct calculations for all investments
4. **Get**: Should display detailed information for each investment
5. **Update**: Should modify investments and save changes
6. **Delete**: Should remove investments from storage
7. **Filters**: Should correctly filter investments by type and tags
8. **JSON**: Should output valid JSON with correct structure
9. **Errors**: Should display appropriate error messages

## Notes

- Test prices are intentionally set to show both gains and losses
- BTC and ETH should show positive returns based on test prices
- Stock prices vary to test profit/loss calculation
- All data is stored locally in `~/.config/i-rs/invest.json`
