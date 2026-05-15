# i-rs-budget Overview

Personal budget management CLI tool for tracking budgets and expenses.

## Core Concepts

### Budget

A budget defines a spending limit for a category:

| Field | Description |
|-------|-------------|
| category | Budget category name |
| amount | Budget amount |
| period | Period (daily/weekly/monthly/yearly) |
| tags | Tag list |
| remark | Notes |

### Expense

An expense is an actual spending record:

| Field | Description |
|-------|-------------|
| id | Unique identifier |
| category | Budget category |
| amount | Expense amount |
| description | Expense description |
| date | Expense date |
| tags | Tag list |

## Quick Start

### 1. Add Budget

```bash
# Add monthly budget
i-rs-budget add food 500

# Add weekly budget
i-rs-budget add groceries 300 --period weekly
```

### 2. Record Expense

```bash
# Record an expense
i-rs-budget expense food 25.50 --description "Lunch"
```

### 3. View Statistics

```bash
# View current month budget usage
i-rs-budget stats
```

## Commands

| Command | Description |
|---------|-------------|
| `add` | Add a new budget |
| `expense` | Record an expense |
| `list` | List budgets or expenses |
| `stats` | View budget statistics |
| `get` | Get budget or expense details |
| `update` | Update budget information |
| `delete` | Delete a budget or expense |
| `example` | Show usage examples |
| `skill` | View AI skill documentation |

## Data Storage

- **Location**: `~/.config/i-rs/budget.json`
- **Format**: JSON
- **Override**: Set `CONFIG_DIR` environment variable for custom path
