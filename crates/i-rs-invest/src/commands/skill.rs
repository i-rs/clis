#[derive(Debug, Clone, Copy)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_CONTENT: &str = r#"# i-rs-invest

Investment returns tracking CLI tool for stocks, funds, and cryptocurrencies.

## Storage

- Config: `~/.config/i-rs/invest.json`

## Commands

### add
Add a new investment:
```bash
i-rs-invest add <name> --symbol <symbol> --type <type> --quantity <qty> --price <price> [--date <date>] [--current-price <price>] [--tag <tag>] [--remark <remark>]
```

Arguments:
- `name`: Investment name (e.g., "Apple")
- `--symbol`: Ticker symbol (e.g., "AAPL")
- `--type`: Asset type (stock, fund, crypto)
- `--quantity`: Number of units held
- `--price`: Buy price per unit
- `--date`: Buy date (optional, defaults to today)
- `--current-price`: Current price per unit (optional)
- `--tag`: Tags for categorization (multiple allowed)
- `--remark`: Additional notes (multiple allowed)

### list
List all investments:
```bash
i-rs-invest list [--type <type>] [--tag <tag>]
```

Options:
- `--type`: Filter by asset type (stock, fund, crypto)
- `--tag`: Filter by tag

### get
Get detailed information about an investment:
```bash
i-rs-invest get <name>
```

### update
Update investment details:
```bash
i-rs-invest update <name> [--symbol <symbol>] [--type <type>] [--quantity <qty>] [--price <price>] [--current-price <price>] [--tag <tags>] [--remark <remarks>]
```

### delete
Delete an investment:
```bash
i-rs-invest delete <name>
```

### stats
View portfolio statistics:
```bash
i-rs-invest stats
```

Shows:
- Total cost and value
- Total profit/loss
- Breakdown by asset type
- Top performers

### example
Show usage examples:
```bash
i-rs-invest example
```

### skill
View skill documentation:
```bash
i-rs-invest skill [summary|content|raw]
```

## Asset Types

- **stock**: Individual stocks (e.g., AAPL, GOOGL)
- **fund**: Mutual funds, ETFs (e.g., VTI, SPY)
- **crypto**: Cryptocurrencies (e.g., BTC, ETH)

## Examples

```bash
# Add a stock
i-rs-invest add Apple --symbol AAPL --type stock --quantity 10 --price 150.00

# Add a fund with tags
i-rs-invest add "S&P 500 Fund" --symbol VOO --type fund --qty 100 --price 400 --tag etf --tag index

# Update current price
i-rs-invest update Apple --current-price 175.50

# View stats
i-rs-invest stats

# List stocks only
i-rs-invest list --type stock

# JSON output
i-rs-invest list --json
```
"#;

const SKILL_SUMMARY: &str = "Investment returns tracking for stocks, funds, and cryptocurrencies with profit/loss calculation and portfolio statistics.";

pub fn handle_skill(command: Option<SkillCommand>) {
    match command {
        Some(SkillCommand::Summary) => {
            println!("{}", SKILL_SUMMARY);
        }
        Some(SkillCommand::Content) => {
            println!("{}", SKILL_CONTENT);
        }
        Some(SkillCommand::Raw) | None => {
            println!("{}", SKILL_CONTENT);
        }
    }
}
