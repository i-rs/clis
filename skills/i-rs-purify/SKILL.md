---
name: "i-rs-purify"
description: "Records water purifier filter replacements. Invoke when user wants to track when they replace water purifier filters."
---

# i-rs-purify

Water purifier filter replacement tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/purify.json`

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

## Examples

```bash
# Record replacement
i-rs-purify add "RO Membrane"
i-rs-purify add "Carbon Filter" --tag kitchen

# List records
i-rs-purify list
```