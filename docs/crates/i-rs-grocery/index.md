# i-rs-grocery

Grocery list CLI tool for managing shopping lists with quantities and purchase tracking.

## Overview

i-rs-grocery helps you manage your shopping list from the terminal. Add items with quantities, mark them as purchased, and filter by tags.

## Quick Start

```bash
# Add items to your grocery list
i-rs-grocery add milk 2 bottles --tag dairy
i-rs-grocery add eggs 1 dozen --tag dairy
i-rs-grocery add bread 1 loaf --tag bakery

# View all items
i-rs-grocery list

# Mark items as purchased
i-rs-grocery purchase milk

# Filter needed items
i-rs-grocery list --needed

# Clear purchased items
i-rs-grocery clear
```

## Key Features

- **Quantity & Unit**: Track items with specific quantities and units
- **Purchase Status**: Mark items as purchased or needed
- **Tag Support**: Organize items with tags
- **Filtering**: Filter by tag or purchase status
- **JSON Output**: Use `--json` flag for programmatic access