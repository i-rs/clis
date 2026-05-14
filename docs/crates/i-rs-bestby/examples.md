# i-rs-bestby Examples

## Basic Usage

### Adding Items

```bash
# Add perishable item with 7-day cycle
i-rs-bestby add "Milk" --purchase-date 2024-01-01 --cycle-days 7

# Add electronics with yearly cycle
i-rs-bestby add "Phone Battery" --purchase-date 2023-06-01 --cycle-days 365

# Add item without cycle (no expiration tracking)
i-rs-bestby add "Winter Jacket" --purchase-date 2023-10-15
```

### Viewing Items

```bash
# List all items
i-rs-bestby list

# Get specific item
i-rs-bestby get Milk
```

## Tagging

```bash
# Add items with tags
i-rs-bestby add "Yogurt" --purchase-date 2024-01-10 --cycle-days 14 --tag dairy --tag breakfast
i-rs-bestby add "Printer Ink" --purchase-date 2024-01-05 --cycle-days 90 --tag office --tag supplies
```

## Managing Items

```bash
# Update replacement cycle
i-rs-bestby update "Milk" --cycle-days 10

# Delete item
i-rs-bestby delete "Old Item"
```

## Household Item Tracking

```bash
# Kitchen
i-rs-bestby add "Cutting Board" --purchase-date 2024-01-01 --cycle-days 365 --tag kitchen
i-rs-bestby add "Sponges" --purchase-date 2024-01-15 --cycle-days 30 --tag kitchen

# Bathroom
i-rs-bestby add "Toothbrush Pack" --purchase-date 2024-01-01 --cycle-days 90 --tag bathroom
i-rs-bestby add "Shampoo" --purchase-date 2024-01-10 --cycle-days 60 --tag bathroom

# Office
i-rs-bestby add "Monitor Cable" --purchase-date 2023-06-01 --cycle-days 730 --tag office
```