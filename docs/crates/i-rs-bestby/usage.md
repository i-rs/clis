# i-rs-bestby Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Add an item with purchase date and replacement cycle.

```bash
i-rs-bestby add <NAME> <PURCHASE_DATE> [OPTIONS]
```

Arguments:
- `NAME` - Item name
- `PURCHASE_DATE` - Purchase date (YYYY-MM-DD)

Options:
- `--cycle-days <DAYS>` - Replacement cycle in days
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List all items.

```bash
i-rs-bestby list
```

### get

Get item details.

```bash
i-rs-bestby get <NAME>
```

### delete

Delete an item.

```bash
i-rs-bestby delete <NAME>
```

### update

Update an item.

```bash
i-rs-bestby update <NAME> [OPTIONS]
```

Options:
- `--cycle-days <DAYS>` - Update replacement cycle
- `--purchase-date <DATE>` - Update purchase date
- `-t, --tag <TAG>` - Add tags
- `-r, --remark <REMARK>` - Add remarks

### data

Manage stored data.

```bash
i-rs-bestby data export
i-rs-bestby data import [FILE]
i-rs-bestby data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data

### example

Show usage examples.

```bash
i-rs-bestby example
```

### skill

Show skill information.

```bash
i-rs-bestby skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/bestby.json`
- Linux: `~/.config/i-rs/bestby.json`
- Windows: `~\AppData\Roaming\i-rs\bestby.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-bestby list
```
