---
name: "i-rs-spark"
description: "Records inspiration and ideas. Invoke when user wants to capture fleeting thoughts or creative sparks."
---

# i-rs-spark

Inspiration and ideas tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/spark.json`

## Commands

### add

Capture inspiration.

```bash
i-rs-spark add <CONTENT> [OPTIONS]
```

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

## Examples

```bash
# Capture inspiration
i-rs-spark add "Use machine learning for text classification"
i-rs-spark add "New app idea" --source "Dream"

# List sparks
i-rs-spark list

# Get details
i-rs-spark get abc12345
```