---
name: "i-rs-walkdog"
description: "Records dog walks. Invoke when user wants to track when they walk their dog."
---

# i-rs-walkdog

Dog walking tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/walkdog.json`

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Record dog walk.

```bash
i-rs-walkdog add <DOG_NAME> <DURATION_MINUTES> [OPTIONS]
```

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

## Examples

```bash
# Record walk
i-rs-walkdog add "Buddy" 30
i-rs-walkdog add "Max" 45 --tag morning

# List records
i-rs-walkdog list
```