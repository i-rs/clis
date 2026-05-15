# i-rs-step Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Record daily steps.

```bash
i-rs-step add <STEPS> <DATE> [OPTIONS]
```

Arguments:
- `STEPS` - Number of steps
- `DATE` - Date (YYYY-MM-DD)

Options:
- `-d, --distance <KM>` - Distance in kilometers
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List step records.

```bash
i-rs-step list
```

### get

Get record details.

```bash
i-rs-step get <DATE>
```

### delete

Delete a record.

```bash
i-rs-step delete <DATE>
```

### update

Update a record.

```bash
i-rs-step update <DATE> [OPTIONS]
```

Options:
- `-s, --steps <STEPS>` - Update steps
- `-d, --distance <KM>` - Update distance
- `-t, --tag <TAG>` - Update tags
- `-r, --remark <REMARK>` - Update remarks

### data

Manage data (export, import, clear).

```bash
i-rs-step data export
i-rs-step data import [FILE]
i-rs-step data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data

### example

Show usage examples.

```bash
i-rs-step example
```

### skill

Show skill information.

```bash
i-rs-step skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/step.json`
- Linux: `~/.config/i-rs/step.json`
- Windows: `~\AppData\Roaming\i-rs\step.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-step list
```
