# i-rs-spark Usage

## Commands

### add

Capture inspiration.

```bash
i-rs-spark add <CONTENT> [OPTIONS]
```

Arguments:
- `CONTENT` - The inspiration or idea

Options:
- `--source <SOURCE>` - Source of inspiration (dream, book, conversation, etc.)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List all sparks.

```bash
i-rs-spark list
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

## Data Storage

- macOS: `~/.config/i-rs/spark.json`
- Linux: `~/.config/i-rs/spark.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-spark list
```