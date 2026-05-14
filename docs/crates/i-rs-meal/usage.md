# i-rs-meal Usage

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

## Data Storage

- macOS: `~/.config/i-rs/meal.json`
- Linux: `~/.config/i-rs/meal.json`
- Windows: `~\AppData\Roaming\i-rs\meal.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-meal list
```