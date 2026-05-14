# i-rs-cal Examples

## Basic Usage

### Recording Meals

```bash
# Fruits
i-rs-cal add "Apple" 95
i-rs-cal add "Banana" 105
i-rs-cal add "Orange" 62

# Main meals
i-rs-cal add "Pizza" 285 --tag lunch
i-rs-cal add "Burger" 350 --tag lunch
i-rs-cal add "Salad" 150 --tag dinner
```

### With Tags

```bash
# Tagged by meal
i-rs-cal add "Toast" 150 --tag breakfast
i-rs-cal add "Eggs" 200 --tag breakfast
i-rs-cal add "Sandwich" 350 --tag lunch
i-rs-cal add "Steak" 500 --tag dinner

# Tagged by type
i-rs-cal add "Chocolate" 230 --tag snack
i-rs-cal add "Chips" 150 --tag snack
```

### With Date

```bash
# Record past intake
i-rs-cal add "Pizza" 285 --date 2024-01-15
i-rs-cal add "Pasta" 400 --date 2024-01-14
```

## Viewing Records

```bash
# List all records
i-rs-cal list

# Filter by tag
i-rs-cal list --tag breakfast
i-rs-cal list --tag lunch
```

## Managing Records

```bash
# Get record details
i-rs-cal get abc12345

# Delete a record
i-rs-cal delete abc12345
```

## Daily Tracking

```bash
# Breakfast
i-rs-cal add "Oatmeal" 150 --tag breakfast
i-rs-cal add "Coffee" 5 --tag breakfast
i-rs-cal add "Banana" 105 --tag breakfast

# Lunch
i-rs-cal add "Sandwich" 350 --tag lunch
i-rs-cal add "Apple" 95 --tag lunch

# Dinner
i-rs-cal add "Chicken Breast" 200 --tag dinner
i-rs-cal add "Rice" 200 --tag dinner
i-rs-cal add "Vegetables" 100 --tag dinner
```