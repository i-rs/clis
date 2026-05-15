# i-rs-walkdog Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Add a dog walk record.

```bash
i-rs-walkdog add <DOG_NAME> <DURATION_MINUTES> [OPTIONS]
```

Arguments:
- `DOG_NAME` - Name of the dog
- `DURATION_MINUTES` - Walk duration in minutes

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List dog walk records.

```bash
i-rs-walkdog list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-walkdog get <ID>
```

### delete

Delete a record.

```bash
i-rs-walkdog delete <ID>
```

### data

Manage data (export, import, clear).

```bash
i-rs-walkdog data export
i-rs-walkdog data import [FILE]
i-rs-walkdog data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
### example

Show usage examples.

```bash
i-rs-walkdog example
```
### skill

Show skill information.

```bash
i-rs-walkdog skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/walkdog.json`
- Linux: `~/.config/i-rs/walkdog.json`
- Windows: `~\AppData\Roaming\i-rs\walkdog.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-walkdog list
```