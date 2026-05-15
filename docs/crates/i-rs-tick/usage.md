# i-rs-tick Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Record task duration.

```bash
i-rs-tick add <TASK_NAME> <SECONDS> [OPTIONS]
```

Arguments:
- `TASK_NAME` - Name of the task
- `SECONDS` - Duration in seconds

Options:
- `-s, --started-at <DATETIME>` - Start time (YYYY-MM-DD HH:MM:SS)
- `-d, --description <DESC>` - Description
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List duration records.

```bash
i-rs-tick list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-tick get <ID>
```

### delete

Delete a record.

```bash
i-rs-tick delete <ID>
```

### data

Manage data (export, import, clear).

```bash
i-rs-tick data export
i-rs-tick data import [FILE]
i-rs-tick data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data

### example

Show usage examples.

```bash
i-rs-tick example
```

### skill

Show skill information.

```bash
i-rs-tick skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/tick.json`
- Linux: `~/.config/i-rs/tick.json`
- Windows: `~\AppData\Roaming\i-rs\tick.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-tick list
```
