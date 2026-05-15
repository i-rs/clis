# i-rs-sleep Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Record a sleep session.

```bash
i-rs-sleep add <BEDTIME> <WAKE_TIME> <QUALITY> [OPTIONS]
```

Arguments:
- `BEDTIME` - Bedtime in HH:MM format
- `WAKE_TIME` - Wake time in HH:MM format
- `QUALITY` - Sleep quality (1-5)

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List all sleep records.

```bash
i-rs-sleep list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get sleep record details.

```bash
i-rs-sleep get <ID>
```

Arguments:
- `ID` - Record ID

### stats

Show sleep statistics.

```bash
i-rs-sleep stats
```

### update

Update a sleep record.

```bash
i-rs-sleep update <ID> [OPTIONS]
```

Options:
- `-b, --bedtime <TIME>` - Update bedtime
- `-w, --wake-time <TIME>` - Update wake time
- `-q, --quality <1-5>` - Update quality
- `-t, --tag <TAG>` - Update tags
- `-r, --remark <REMARK>` - Update remarks

### delete

Delete a sleep record.

```bash
i-rs-sleep delete <ID>
```

Arguments:
- `ID` - Record ID

## Quality Scale

| Rating | Emoji | Description |
|--------|-------|-------------|
| 1 | 😴 | Awful |
| 2 | 😪 | Poor |
| 3 | 😌 | Fair |
| 4 | 😊 | Good |
| 5 | 😁 | Excellent |

### data

Manage data (export, import, clear).

```bash
i-rs-sleep data export
i-rs-sleep data import [FILE]
i-rs-sleep data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
### example

Show usage examples.

```bash
i-rs-sleep example
```
### skill

Show skill information.

```bash
i-rs-sleep skill [summary|content|raw]
```

## Data Storage

- macOS: `~/Library/Application Support/i-rs/sleep.json`
- Linux: `~/.config/i-rs/sleep.json`
- Windows: `~\AppData\Roaming\i-rs\sleep.json`