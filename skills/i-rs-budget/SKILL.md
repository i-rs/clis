---
name: "i-rs-budget"
description: "Personal budget management tool. Set budgets, record expenses, and view statistics. Invoke when user needs to track daily spending, manage category budgets, or view budget analytics."
---

# i-rs-budget

Personal budget management CLI tool for setting budget categories, recording expenses, and viewing statistics.

## Storage

- **Config file**: `~/.config/i-rs/budget.json`
- **Environment**: `CONFIG_DIR` overrides default path
- **Data format**: JSON with budgets and expenses collections

## Global Flags

- `--json` — Output in JSON format

## Core Concepts

### Budget

Define a spending limit for a category:

- `category`: Budget category name
- `amount`: Budget amount
- `period`: Period (daily/weekly/monthly/yearly)
- `tags`: Tag list
- `remark`: Notes

### Expense

Record actual spending:

- `id`: Unique identifier (UUID first 8 chars)
- `category`: Budget category
- `amount`: Expense amount
- `description`: Expense description
- `date`: Expense date
- `tags`: Tag list

## Commands

### add

Add a new budget:

```bash
i-rs-budget add <CATEGORY> <AMOUNT> [OPTIONS]
```

Options:
- `-p, --period <PERIOD>` - Budget period: daily, weekly, monthly, yearly
- `-t, --tags <TAGS>` - Tag list
- `-r, --remark <REMARK>` - Remarks

Examples:
```bash
i-rs-budget add food 500                              # Monthly budget
i-rs-budget add groceries 300 --period weekly         # Weekly budget
i-rs-budget add vacation 5000 --period yearly         # Yearly budget
i-rs-budget add entertainment 200 --tags fun,leisure  # With tags
```

### expense

Record an expense:

```bash
i-rs-budget expense <CATEGORY> <AMOUNT> [OPTIONS]
```

Options:
- `-d, --description <DESC>` - Expense description (required)
- `--date <DATE>` - Expense date, format: YYYY-MM-DD
- `-t, --tags <TAGS>` - Tag list

Examples:
```bash
i-rs-budget expense food 25.50 --description "Lunch"
i-rs-budget expense groceries 120.00 --date 2024-01-15
i-rs-budget expense entertainment 60.00 --tags movie
```

### list

List budgets or expenses:

```bash
i-rs-budget list [TYPE] [OPTIONS]
```

Options:
- `-c, --category <CAT>` - Filter by category

Examples:
```bash
i-rs-budget list budgets                              # List all budgets
i-rs-budget list expenses                             # List all expenses
i-rs-budget list expenses --category food             # Filter by category
```

### stats

View budget statistics:

```bash
i-rs-budget stats [OPTIONS]
```

Options:
- `-c, --category <CAT>` - Filter by category
- `-p, --period <PERIOD>` - Stats period: daily, weekly, monthly, yearly

Examples:
```bash
i-rs-budget stats                                     # Current month stats
i-rs-budget stats --period weekly                     # Current week stats
i-rs-budget stats --category food                    # Specific category
i-rs-budget stats --category food --period monthly    # Combined filter
```

### get

Get budget or expense details:

```bash
i-rs-budget get [OPTIONS]
```

Options:
- `-c, --category <CAT>` - Get specific budget category
- `--expense-id <ID>` - Get specific expense record

### update

Update budget information:

```bash
i-rs-budget update <CATEGORY> [OPTIONS]
```

Options:
- `-a, --amount <AMOUNT>` - New budget amount
- `-p, --period <PERIOD>` - New budget period
- `-t, --tags <TAGS>` - New tag list
- `-r, --remark <REMARK>` - New remarks

Examples:
```bash
i-rs-budget update food --amount 600                  # Update amount
i-rs-budget update food --period weekly              # Update period
i-rs-budget update food --tags essentials             # Update tags
```

### delete

Delete a budget or expense:

```bash
i-rs-budget delete [OPTIONS]
```

Options:
- `-c, --category <CAT>` - Delete budget category (includes all related expenses)
- `--expense-id <ID>` - Delete single expense

### data

Manage data (export, import, clear).

```bash
i-rs-budget data export
i-rs-budget data import [FILE]
i-rs-budget data clear
```

### example

Show usage examples:

```bash
i-rs-budget example
```

### skill

View AI skill documentation:

```bash
i-rs-budget skill [summary|content|raw]
```

## Common Scenarios

### Monthly Budget Tracking

```bash
# Set monthly budgets
i-rs-budget add food 1000 --period monthly [OPTIONS]
i-rs-budget add groceries 500 --period monthly [OPTIONS]
i-rs-budget add entertainment 300 --period monthly [OPTIONS]

# Daily tracking
i-rs-budget expense food 35.00 --description "Lunch"
i-rs-budget expense groceries 150.00 --description "Supermarket"

# Check progress
i-rs-budget stats --period monthly
```

### Weekly Budget Control

```bash
# Set weekly budget
i-rs-budget add groceries 300 --period weekly [OPTIONS]

# Record during the week
i-rs-budget expense groceries 80.00 --description "Monday shopping"
i-rs-budget expense groceries 120.00 --description "Wednesday restock"

# Check remaining
i-rs-budget stats --period weekly --category groceries
```

### Spending Analysis

```bash
# View all expenses
i-rs-budget list expenses

# Analyze by category
i-rs-budget stats --category food
i-rs-budget stats --category entertainment

# Yearly overview
i-rs-budget stats --period yearly
```
