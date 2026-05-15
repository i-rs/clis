---
name: "i-rs-recur"
description: "Tracks recurring expenses. Invoke when user wants to manage fixed expenses like rent, subscriptions, or bills."
---

# i-rs-recur

Recurring expenses tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/recur.json`

## Global Flags

- `--json` — Output in JSON format
## Commands

### add

Add a recurring expense.

```bash
i-rs-recur add <NAME> --amount <AMOUNT> --frequency <FREQ> [OPTIONS]
```

Options:
- `--amount <AMOUNT>` - Amount
- `--currency <CURRENCY>` - Currency (default: CNY)
- `--frequency <FREQ>` - Frequency (daily, weekly, monthly, quarterly, yearly)
- `--start-date <DATE>` - Start date (YYYY-MM-DD)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List recurring expenses.

```bash
i-rs-recur list [OPTIONS]
```

### get

Get expense details.

```bash
i-rs-recur get <NAME>
```

### delete

Delete an expense.

```bash
i-rs-recur delete <NAME>
```

### update

Update an expense.

```bash
i-rs-recur update <NAME> [OPTIONS]
```

Options:
- `--amount <AMOUNT>` - Update amount
- `--frequency <FREQ>` - Update frequency
- `-t, --tag <TAG>` - Add tags
- `-r, --remark <REMARK>` - Add remarks

### data

Manage data (export, import, clear).

```bash
i-rs-recur data export
i-rs-recur data import [FILE]
i-rs-recur data clear
```

### example

Show usage examples.

```bash
i-rs-recur example
```

### skill

Show skill information.

```bash
i-rs-recur skill [summary|content|raw]
```

## Examples

```bash
# Add recurring expense
i-rs-recur add "Netflix" --amount 15.99 --frequency monthly [OPTIONS]
i-rs-recur add "Rent" --amount 2000 --frequency monthly [OPTIONS]
i-rs-recur add "Gym" --amount 300 --frequency monthly [OPTIONS]

# List expenses
i-rs-recur list [OPTIONS]

# Get details
i-rs-recur get Netflix
```