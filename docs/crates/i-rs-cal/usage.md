# i-rs-cal Usage

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

## Data Storage

- macOS: `~/.config/i-rs/cal.json`
- Linux: `~/.config/i-rs/cal.json`
- Windows: `~\AppData\Roaming\i-rs\cal.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-cal list
```