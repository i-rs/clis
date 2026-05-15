---
name: "i-rs-grocery"
description: "Manages grocery shopping list with quantities and purchase tracking. Invoke when user wants to create or manage a shopping list."
---

# i-rs-grocery

Grocery list CLI tool for managing shopping lists with quantities and purchase tracking.

## Storage

- Config: `~/.config/i-rs/grocery.json`

## Global Flags

- `--json` — Output in JSON format
## Commands

### add

Add an item to the grocery list.

```bash
i-rs-grocery add <NAME> [QUANTITY] [UNIT] [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List grocery items.

```bash
i-rs-grocery list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag
- `--purchased` - Show only purchased items
- `-n, --needed` - Show only needed items

### purchase

Toggle purchase status of an item.

```bash
i-rs-grocery purchase <NAME>
```

### clear

Clear all purchased items.

```bash
i-rs-grocery clear
```

### get

Get item details.

```bash
i-rs-grocery get <NAME>
```

### update

Update an item.

```bash
i-rs-grocery update <NAME> [OPTIONS]
```

### delete

Delete an item.

```bash
i-rs-grocery delete <NAME>
```

### data

Manage data (export, import, clear).

```bash
i-rs-grocery data export
i-rs-grocery data import [FILE]
i-rs-grocery data clear
```

### example

Show usage examples.

```bash
i-rs-grocery example
```

### skill

Show skill information.

```bash
i-rs-grocery skill [summary|content|raw]
```

## Examples

```bash
# Add items
i-rs-grocery add milk 2 bottles --tag dairy [OPTIONS]
i-rs-grocery add eggs 1 dozen --tag dairy [OPTIONS]

# List items
i-rs-grocery list [OPTIONS]
i-rs-grocery list --needed

# Mark as purchased
i-rs-grocery purchase milk

# Clear purchased items
i-rs-grocery clear
```

## Common Units

- item, kg, g, liter, bottle, pack, dozen, loaf, box