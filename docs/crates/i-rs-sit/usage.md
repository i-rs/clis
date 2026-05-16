# i-rs-sit Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Add a sitting duration record.

```bash
i-rs-sit add <DURATION_MINUTES> [OPTIONS]
```

Arguments:
- `DURATION_MINUTES` - Duration in minutes

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List sitting records.

```bash
i-rs-sit list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-sit get <ID>
```

### delete

Delete a record.

```bash
i-rs-sit delete <ID>
```

### update

Update a sitting record.

```bash
i-rs-sit update <ID> [OPTIONS]
```

Options:
- `--duration-minutes <MIN>` - Duration in minutes
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### data

Manage data (export, import, clear).

```bash
i-rs-sit data export
i-rs-sit data import [FILE]
i-rs-sit data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
### example

Show usage examples.

```bash
i-rs-sit example
```
### skill

Show skill information.

```bash
i-rs-sit skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/sit.json`
- Linux: `~/.config/i-rs/sit.json`
- Windows: `~\AppData\Roaming\i-rs\sit.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-sit list
```