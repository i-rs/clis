---
name: "i-rs-petbath"
description: "Records pet baths. Invoke when user wants to track when they bathe their pets."
---

# i-rs-petbath

Pet bath tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/petbath.json`

## Global Flags

- `--json` — Output in JSON format
## Commands

### add

Record pet bath.

```bash
i-rs-petbath add <PET_NAME>
```

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List pet bath records.

```bash
i-rs-petbath list
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-petbath get <ID>
```

### delete

Delete a record.

```bash
i-rs-petbath delete <ID>
```

### data

Manage data (export, import, clear).

```bash
i-rs-petbath data export
i-rs-petbath data import [FILE]
i-rs-petbath data clear
```

### example

Show usage examples.

```bash
i-rs-petbath example
```

### skill

Show skill information.

```bash
i-rs-petbath skill [summary|content|raw]
```

## Examples

```bash
# Record bath
i-rs-petbath add "Cat"
i-rs-petbath add "Dog" --tag summer

# List records
i-rs-petbath list
```