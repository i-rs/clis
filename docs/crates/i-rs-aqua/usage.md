# i-rs-aqua Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Add a water change record.

```bash
i-rs-aqua add [OPTIONS]
```

Options:
- `-t, --tank-size <SIZE>` - Tank size in liters
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List water change records.

```bash
i-rs-aqua list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-aqua get <ID>
```

### delete

Delete a record.

```bash
i-rs-aqua delete <ID>
```

### data

Manage data (export, import, clear).

```bash
i-rs-aqua data export
i-rs-aqua data import [FILE]
i-rs-aqua data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data

### example

Show usage examples.

```bash
i-rs-aqua example
```

### skill

Show skill information.

```bash
i-rs-aqua skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/aqua.json`
- Linux: `~/.config/i-rs/aqua.json`
- Windows: `~\AppData\Roaming\i-rs\aqua.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-aqua list
```