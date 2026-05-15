# i-rs-cal Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Add a calorie record.

```bash
i-rs-cal add <FOOD_NAME> <CALORIES> [OPTIONS]
```

Arguments:
- `FOOD_NAME` - Name of the food
- `CALORIES` - Calorie amount

Options:
- `-d, --date <DATE>` - Date in YYYY-MM-DD format (defaults to today)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List calorie records.

```bash
i-rs-cal list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-cal get <ID>
```

### delete

Delete a record.

```bash
i-rs-cal delete <ID>
```

### data

Manage data (export, import, clear).

```bash
i-rs-cal data export
i-rs-cal data import [FILE]
i-rs-cal data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
### example

Show usage examples.

```bash
i-rs-cal example
```
### skill

Show skill information.

```bash
i-rs-cal skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/cal.json`
- Linux: `~/.config/i-rs/cal.json`
- Windows: `~\AppData\Roaming\i-rs\cal.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-cal list
```