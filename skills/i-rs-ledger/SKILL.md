---
name: "i-rs-ledger"
description: "Tracks income, expenses, and transfers for personal accounting. Invoke when user wants to record financial transactions."
---

# i-rs-ledger

Personal accounting CLI tool.

## Storage

- Config: `~/.config/i-rs/ledger.json`

## Global Flags

- `--json` — Output in JSON format
## Commands

### add

Add a ledger entry.

```bash
i-rs-ledger add [OPTIONS]
```

Options:
- `--type <TYPE>` - Entry type (income, expense, transfer)
- `--amount <AMOUNT>` - Amount
- `--currency <CURRENCY>` - Currency (default: CNY)
- `--category <CATEGORY>` - Category
- `--date <DATE>` - Date (YYYY-MM-DD)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List ledger entries.

```bash
i-rs-ledger list
```

Options:
- `--type <TYPE>` - Filter by type
- `--category <CATEGORY>` - Filter by category

### get

Get entry details.

```bash
i-rs-ledger get <ID>
```

### delete

Delete an entry.

```bash
i-rs-ledger delete <ID>
```

### data

Manage data (export, import, clear).

```bash
i-rs-ledger data export
i-rs-ledger data import [FILE]
i-rs-ledger data clear
```

### example

Show usage examples.

```bash
i-rs-ledger example
```

### skill

Show skill information.

```bash
i-rs-ledger skill [summary|content|raw]
```

## Examples

```bash
# Add income
i-rs-ledger add --type income --amount 5000 --category salary

# Add expense
i-rs-ledger add --type expense --amount 150 --category food

# Add transfer
i-rs-ledger add --type transfer --amount 1000 --category savings

# List entries
i-rs-ledger list
```