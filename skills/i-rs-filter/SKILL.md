---
name: "i-rs-filter"
description: "Records appliance filter cleaning. Invoke when user wants to track when they clean appliance filters."
---

# i-rs-filter

Appliance filter cleaning tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/filters.json`

## Commands

### add

Record filter cleaning.

```bash
i-rs-filter add <APPLIANCE_NAME> <FILTER_TYPE>
```

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List filter cleaning records.

```bash
i-rs-filter list
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-filter get <ID>
```

### delete

Delete a record.

```bash
i-rs-filter delete <ID>
```

## Filter Types

- `HEPA` - High-efficiency particulate air
- `Carbon` - Activated carbon filter
- `Foam` - Foam filter
- `Dust` - Dust collection filter

## Examples

```bash
# Record cleaning
i-rs-filter add "Air Purifier" HEPA
i-rs-filter add "Vacuum" dust --tag living-room

# List records
i-rs-filter list
```