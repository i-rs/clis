---
name: "i-rs-grocery"
description: "Manages grocery shopping list with quantities and purchase tracking. Invoke when user wants to create or manage a shopping list."
---

# i-rs-grocery

Grocery list CLI tool for managing shopping lists with quantities and purchase tracking.

## Storage

- Config: `~/.config/i-rs/grocery.json`

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

## Examples

```bash
# Add items
i-rs-grocery add milk 2 bottles --tag dairy
i-rs-grocery add eggs 1 dozen --tag dairy

# List items
i-rs-grocery list
i-rs-grocery list --needed

# Mark as purchased
i-rs-grocery purchase milk

# Clear purchased items
i-rs-grocery clear
```

## Common Units

- item, kg, g, liter, bottle, pack, dozen, loaf, box