# i-rs-fast Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Start a fasting session.

```bash
i-rs-fast add <TARGET_HOURS> [OPTIONS]
```

Arguments:
- `TARGET_HOURS` - Target fasting duration in hours

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List fasting records.

```bash
i-rs-fast list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-fast get <ID>
```

### delete

Delete a record.

```bash
i-rs-fast delete <ID>
```

## Fasting Protocols

| Protocol | Fasting Hours | Eating Window |
|----------|--------------|---------------|
| 16:8 | 16 hours | 8 hours |
| 18:6 | 18 hours | 6 hours |
| 20:4 | 20 hours | 4 hours |
| OMAD | 23 hours | 1 hour |

### data

Manage data (export, import, clear).

```bash
i-rs-fast data export
i-rs-fast data import [FILE]
i-rs-fast data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
### example

Show usage examples.

```bash
i-rs-fast example
```
### skill

Show skill information.

```bash
i-rs-fast skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/fast.json`
- Linux: `~/.config/i-rs/fast.json`
- Windows: `~\AppData\Roaming\i-rs\fast.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-fast list
```