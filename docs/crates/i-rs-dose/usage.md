# i-rs-dose Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Record medicine intake.

```bash
i-rs-dose add <MEDICINE_NAME> <DOSAGE> <UNIT> [OPTIONS]
```

Arguments:
- `MEDICINE_NAME` - Name of the medicine
- `DOSAGE` - Dosage amount
- `UNIT` - Unit (tablet, ml, mg, IU, drop, capsule, etc.)

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List medicine records.

```bash
i-rs-dose list
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-dose get <ID>
```

### delete

Delete a record.

```bash
i-rs-dose delete <ID>
```

### data

Manage stored data.

```bash
i-rs-dose data export
i-rs-dose data import [FILE]
i-rs-dose data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data

### example

Show usage examples.

```bash
i-rs-dose example
```

### skill

Show skill information.

```bash
i-rs-dose skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/dose.json`
- Linux: `~/.config/i-rs/dose.json`
- Windows: `~\AppData\Roaming\i-rs\dose.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-dose list
```
