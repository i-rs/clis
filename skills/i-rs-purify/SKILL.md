---
name: "i-rs-purify"
description: "Records water purifier filter replacements. Invoke when user wants to track when they replace water purifier filters."
---

# i-rs-purify

Water purifier filter replacement tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/purify.json`

## Global Flags

- `--json` — Output in JSON format
## Commands

### add

Record filter replacement.

```bash
i-rs-purify add <FILTER_TYPE>
```

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List filter replacement records.

```bash
i-rs-purify list
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-purify get <ID>
```

### delete

Delete a record.

```bash
i-rs-purify delete <ID>
```

## Filter Types

- `RO Membrane` - Reverse osmosis membrane
- `Carbon Filter` - Activated carbon filter
- `Sediment Filter` - Pre-filter for sediment
- `Mineral Filter` - Post-filter adding minerals

### data

Manage data (export, import, clear).

```bash
i-rs-purify data export
i-rs-purify data import [FILE]
i-rs-purify data clear
```

### example

Show usage examples.

```bash
i-rs-purify example
```

### skill

Show skill information.

```bash
i-rs-purify skill [summary|content|raw]
```

## Examples

```bash
# Record replacement
i-rs-purify add "RO Membrane"
i-rs-purify add "Carbon Filter" --tag kitchen

# List records
i-rs-purify list
```