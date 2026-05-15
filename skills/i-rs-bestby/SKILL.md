---
name: "i-rs-bestby"
description: "Tracks item best-by dates and replacement cycles. Invoke when user wants to track when items were purchased and when they need to be replaced."
---

# i-rs-bestby

Item best-by date tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/bestby.json`

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Add an item with purchase date and replacement cycle.

```bash
i-rs-bestby add <NAME> <PURCHASE_DATE> [OPTIONS]
```

Arguments:
- `NAME` - Item name
- `PURCHASE_DATE` - Purchase date (YYYY-MM-DD)

Options:
- `--cycle-days <DAYS>` - Replacement cycle in days
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List all items.

```bash
i-rs-bestby list
```

### get

Get item details.

```bash
i-rs-bestby get <NAME>
```

### delete

Delete an item.

```bash
i-rs-bestby delete <NAME>
```

### update

Update an item.

```bash
i-rs-bestby update <NAME> [OPTIONS]
```

Options:
- `--cycle-days <DAYS>` - Update replacement cycle
- `-t, --tag <TAG>` - Add tags
- `-r, --remark <REMARK>` - Add remarks

### data

Manage data (export, import, clear).

```bash
i-rs-bestby data export
i-rs-bestby data import [FILE]
i-rs-bestby data clear
```

### example

Show usage examples.

```bash
i-rs-bestby example
```

### skill

Show skill information.

```bash
i-rs-bestby skill [summary|content|raw]
```

## Examples

```bash
# Add item with cycle
i-rs-bestby add "Milk" 2024-01-01 --cycle-days 7

# Add item without cycle
i-rs-bestby add "Phone Battery" 2023-06-01

# List items
i-rs-bestby list

# Get details
i-rs-bestby get Milk

# JSON output
i-rs-bestby list --json
```
