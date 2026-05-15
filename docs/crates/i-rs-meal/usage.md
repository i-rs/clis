# i-rs-meal Usage

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

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
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

## Data Storage

- macOS: `~/.config/i-rs/meal.json`
- Linux: `~/.config/i-rs/meal.json`
- Windows: `~\AppData\Roaming\i-rs\meal.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-meal list
```