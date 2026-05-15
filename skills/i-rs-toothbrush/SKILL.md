---
name: "i-rs-toothbrush"
description: "Records toothbrush replacements. Invoke when user wants to track when they replace toothbrushes."
---

# i-rs-toothbrush

Toothbrush replacement tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/toothbrushes.json`

## Global Flags

- `--json` — Output in JSON format
## Commands

### add

Record toothbrush replacement.

```bash
i-rs-toothbrush add <BRUSH_TYPE>
```

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List toothbrush replacement records.

```bash
i-rs-toothbrush list
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-toothbrush get <ID>
```

### delete

Delete a record.

```bash
i-rs-toothbrush delete <ID>
```

## Brush Types

- `Electric` - Electric toothbrush head
- `Manual` - Regular manual toothbrush
- `Kids` - Children's toothbrush
- `Interdental` - Interdental brush

### data

Manage data (export, import, clear).

```bash
i-rs-toothbrush data export
i-rs-toothbrush data import [FILE]
i-rs-toothbrush data clear
```

### example

Show usage examples.

```bash
i-rs-toothbrush example
```

### skill

Show skill information.

```bash
i-rs-toothbrush skill [summary|content|raw]
```

## Examples

```bash
# Record replacement
i-rs-toothbrush add "Electric"
i-rs-toothbrush add "Manual" --tag travel

# List records
i-rs-toothbrush list
```