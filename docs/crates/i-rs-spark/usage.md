# i-rs-spark Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Capture inspiration.

```bash
i-rs-spark add <CONTENT> [OPTIONS]
```

Arguments:
- `CONTENT` - The inspiration or idea

Options:
- `-s, --source <SOURCE>` - Source of inspiration (dream, book, conversation, etc.)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List all sparks.

```bash
i-rs-spark list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get spark details.

```bash
i-rs-spark get <ID>
```

### delete

Delete a spark.

```bash
i-rs-spark delete <ID>
```

### update

Update a spark record.

```bash
i-rs-spark update <ID> [OPTIONS]
```

Options:
- `--content <CONTENT>` - The inspiration or idea
- `-s, --source <SOURCE>` - Source of inspiration
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### data

Manage data (export, import, clear).

```bash
i-rs-spark data export
i-rs-spark data import [FILE]
i-rs-spark data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data

### example

Show usage examples.

```bash
i-rs-spark example
```

### skill

Show skill information.

```bash
i-rs-spark skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/spark.json`
- Linux: `~/.config/i-rs/spark.json`
- Windows: `~\AppData\Roaming\i-rs\spark.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-spark list
```