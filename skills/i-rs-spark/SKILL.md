---
name: "i-rs-spark"
description: "Records inspiration and ideas. Invoke when user wants to capture fleeting thoughts or creative sparks."
---

# i-rs-spark

Inspiration and ideas tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/spark.json`

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Capture inspiration.

```bash
i-rs-spark add <CONTENT> [OPTIONS]
```

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

## Examples

```bash
# Capture inspiration
i-rs-spark add "Use machine learning for text classification" [OPTIONS]
i-rs-spark add "New app idea" --source "Dream" [OPTIONS]

# List sparks
i-rs-spark list [OPTIONS]

# Get details
i-rs-spark get abc12345
```