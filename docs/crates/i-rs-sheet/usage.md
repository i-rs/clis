# i-rs-sheet Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Add a sheet change record.

```bash
i-rs-sheet add <SHEET_TYPE> [OPTIONS]
```

Arguments:
- `SHEET_TYPE` - Type of sheet (bedsheet, pillowcase, duvet-cover, mattress-protector)

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List sheet change records.

```bash
i-rs-sheet list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-sheet get <ID>
```

### delete

Delete a record.

```bash
i-rs-sheet delete <ID>
```

## Sheet Types

| Type | Description |
|------|-------------|
| bedsheet | Bottom or top sheet |
| pillowcase | Pillow cover |
| duvet-cover | Comforter/duvet cover |
| mattress-protector | Mattress protector |

### data

Manage data (export, import, clear).

```bash
i-rs-sheet data export
i-rs-sheet data import [FILE]
i-rs-sheet data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
### example

Show usage examples.

```bash
i-rs-sheet example
```
### skill

Show skill information.

```bash
i-rs-sheet skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/sheets.json`
- Linux: `~/.config/i-rs/sheets.json`
- Windows: `~\AppData\Roaming\i-rs\sheet.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-sheet list
```