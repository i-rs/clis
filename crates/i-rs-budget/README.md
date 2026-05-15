# i-rs-budget

Personal budget management CLI tool for setting budgets, recording expenses, and viewing statistics.

## Features

- **Budget Management**: Create, update, delete budgets with different periods (daily/weekly/monthly/yearly)
- **Expense Tracking**: Record expenses with amount, description, date, and tags
- **Statistics**: View budget usage, remaining amounts, and completion percentages
- **Category Analysis**: View expense details by category
- **Period Filtering**: Filter statistics by day/week/month/year
- **JSON Output**: `--json` global flag for programmatic use

## Install

```bash
cargo install i-rs-budget
# or
brew install i-rs/homebrew-tap/i-rs-budget
```

## Quick Start

### Add Budget

```bash
# Add monthly budget (default)
i-rs-budget add food 500

# Add budgets with different periods
i-rs-budget add groceries 300 --period weekly
i-rs-budget add rent 2000 --period monthly
i-rs-budget add vacation 5000 --period yearly

# Add budget with tags
i-rs-budget add entertainment 200 --tags fun,leisure
```

### Record Expense

```bash
# Record expense
i-rs-budget expense food 25.50 --description "Lunch"
i-rs-budget expense groceries 120.30 --date 2024-01-15

# Record with tags
i-rs-budget expense entertainment 50 --description "Movie" --tags movie
```

### View Budgets and Expenses

```bash
# List all budgets
i-rs-budget list budgets

# List all expenses
i-rs-budget list expenses

# View by category
i-rs-budget list expenses --category food
```

### View Statistics

```bash
# View current month stats
i-rs-budget stats

# View by period
i-rs-budget stats --period weekly
i-rs-budget stats --period yearly

# View by category
i-rs-budget stats --category food
```

### Update and Delete

```bash
# Update budget
i-rs-budget update food --amount 600

# Delete budget
i-rs-budget delete --category food

# Delete single expense
i-rs-budget delete --expense-id abc12345
```

## Data Storage

- macOS: `~/.config/i-rs/budget.json`
- Linux: `~/.config/i-rs/budget.json`
- Windows: `~\AppData\Roaming\i-rs\budget.json`

Override with `CONFIG_DIR` environment variable.

## License

MIT OR Apache-2.0
