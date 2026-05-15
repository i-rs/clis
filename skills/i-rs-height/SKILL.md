---
name: "i-rs-height"
description: "Tracks body height (add/list/get/delete). Invoke when user needs to record height measurements, view height history, or display statistics and charts."
---

# i-rs-height

Height tracking CLI tool for monitoring body height and weight over time.

## Storage

- Config: `~/.config/i-rs/heights.json`

## Global Flags

- `--json` — Output in JSON format
## Commands

### add

Add a height record.

```bash
i-rs-height add <DATE> <HEIGHT> [--weight] [--tag] [--remark]
```

### list

List height records.

```bash
i-rs-height list [--days N] [--chart] [--stats]
```

### get

Get specific record.

```bash
i-rs-height get <DATE>
```

### delete

Delete a height record.

```bash
i-rs-height delete <DATE>
```

### set

Set target height.

```bash
i-rs-height set <HEIGHT>
```

### target

Show current target.

```bash
i-rs-height target
```

### data

Manage data (export, import, clear).

```bash
i-rs-height data export
i-rs-height data import [FILE]
i-rs-height data clear
```

### example

Show usage examples.

```bash
i-rs-height example
```

### skill

Show skill information.

```bash
i-rs-height skill [summary|content|raw]
```

## Examples

```bash
# Add record
i-rs-height add 2025-06-14 175.5 [OPTIONS]

# Add with weight
i-rs-height add 2025-06-14 175.5 --weight 68.5 [OPTIONS]

# List with chart
i-rs-height list --chart

# List with stats
i-rs-height list --stats

# Set target
i-rs-height set 180.0
```
