# i-rs-grocery Examples

## Basic Usage

### Adding Items

```bash
# Add items with quantity and unit
i-rs-grocery add milk 2 bottles --tag dairy
i-rs-grocery add eggs 1 dozen --tag dairy
i-rs-grocery add bread 1 loaf --tag bakery

# Add without quantity (defaults to 1 item)
i-rs-grocery add bananas --tag fruit

# Add with remarks
i-rs-grocery add coffee 1 kg --tag pantry --remark "Whole bean"
```

### Viewing Items

```bash
# List all items
i-rs-grocery list

# Filter by tag
i-rs-grocery list --tag dairy

# Show only purchased items
i-rs-grocery list --purchased

# Show only needed items
i-rs-grocery list --needed

# Get detailed info
i-rs-grocery get milk

# Get JSON output
i-rs-grocery list --json
```

### Managing Purchase Status

```bash
# Mark item as purchased
i-rs-grocery purchase milk

# Toggle back to needed
i-rs-grocery purchase milk

# Clear all purchased items
i-rs-grocery clear
```

### Updating Items

```bash
# Update quantity
i-rs-grocery update milk --quantity 3

# Update unit
i-rs-grocery update milk --unit liters

# Update tags
i-rs-grocery update milk --tag dairy --tag fridge
```

### Deleting Items

```bash
# Delete an item
i-rs-grocery delete milk
```

## Real-world Scenarios

### Weekly Grocery List

```bash
# Create weekly grocery list
i-rs-grocery add milk 2 liters --tag dairy
i-rs-grocery add eggs 1 dozen --tag dairy
i-rs-grocery add bread 2 loaves --tag bakery
i-rs-grocery add apples 1 kg --tag fruit
i-rs-grocery add carrots 500g --tag vegetable
i-rs-grocery add chicken 1 kg --tag meat

# View list before shopping
i-rs-grocery list --needed

# Mark items as purchased while shopping
i-rs-grocery purchase milk
i-rs-grocery purchase eggs
i-rs-grocery purchase bread

# After shopping, clear purchased items
i-rs-grocery clear
```

## Output Examples

### List Output

```
╭──────────┬─────┬────────┬─────────────┬────────┬──────────────────╮
│ NAME     │ QTY │ UNIT   │ STATUS      │ TAGS   │ UPDATED          │
├──────────┼─────┼────────┼─────────────┼────────┼──────────────────┤
│ milk     │ 2   │ bottles│ ✅ Purchased│ dairy  │ 2024-01-15 10:30 │
│ eggs     │ 1   │ dozen  │ 🔄 Needed   │ dairy  │ 2024-01-15 10:25 │
│ bread    │ 1   │ loaf   │ 🔄 Needed   │ bakery │ 2024-01-15 10:20 │
╰──────────┴─────┴────────┴─────────────┴────────┴──────────────────╯

Total: 3 items
```