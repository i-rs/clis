---
name: "i-rs-meal"
description: "Records daily meals. Invoke when user wants to track what they eat for breakfast, lunch, dinner, or snacks."
---

# i-rs-meal

Daily meal tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/meal.json`

## Global Flags

- `--json` — Output in JSON format
## Commands

### add

Add a meal record.

```bash
i-rs-meal add <MEAL_TYPE> --food <FOOD_ITEMS> [OPTIONS]
```

Arguments:
- `MEAL_TYPE` - Meal type (breakfast, lunch, dinner, snack)

Options:
- `--food <FOOD_ITEMS>` - Food items description
- `--calories <CALORIES>` - Calorie count
- `--date <DATE>` - Date (YYYY-MM-DD, default: today)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List meal records.

```bash
i-rs-meal list
```

Options:
- `--date <DATE>` - Filter by date
- `-t, --tag <TAG>` - Filter by tag

### get

Get meal details.

```bash
i-rs-meal get <ID>
```

### delete

Delete a meal record.

```bash
i-rs-meal delete <ID>
```

### data

Manage data (export, import, clear).

```bash
i-rs-meal data export
i-rs-meal data import [FILE]
i-rs-meal data clear
```

### example

Show usage examples.

```bash
i-rs-meal example
```

### skill

Show skill information.

```bash
i-rs-meal skill [summary|content|raw]
```

## Examples

```bash
# Record meals
i-rs-meal add breakfast --food "Oatmeal with berries"
i-rs-meal add lunch --food "Salad with chicken"
i-rs-meal add dinner --food "Pasta with seafood"

# With calories
i-rs-meal add lunch --food "Grilled fish" --calories 350

# List meals
i-rs-meal list
```