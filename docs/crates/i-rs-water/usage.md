# i-rs-water Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Add a water intake record.

```bash
i-rs-water add <AMOUNT_ML> [OPTIONS]
```

Arguments:
- `AMOUNT_ML` - Water amount in milliliters

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List water intake records.

```bash
i-rs-water list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-water get <ID>
```

### delete

Delete a record.

```bash
i-rs-water delete <ID>
```

### data

Manage data (export, import, clear).

```bash
i-rs-water data export
i-rs-water data import [FILE]
i-rs-water data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
### example

Show usage examples.

```bash
i-rs-water example
```
### skill

Show skill information.

```bash
i-rs-water skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/water.json`
- Linux: `~/.config/i-rs/water.json`
- Windows: `~\AppData\Roaming\i-rs\water.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-water list
```