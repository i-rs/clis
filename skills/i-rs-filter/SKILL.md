---
name: "i-rs-filter"
description: "Records appliance filter cleaning. Invoke when user wants to track when they clean appliance filters."
---

# i-rs-filter

Appliance filter cleaning tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/filters.json`

## Global Flags

- `--json` — Output in JSON format
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

### data

Manage data (export, import, clear).

```bash
i-rs-filter data export
i-rs-filter data import [FILE]
i-rs-filter data clear
```

### example

Show usage examples.

```bash
i-rs-filter example
```

### skill

Show skill information.

```bash
i-rs-filter skill [summary|content|raw]
```

## Examples

```bash
# Record cleaning
i-rs-filter add "Air Purifier" HEPA
i-rs-filter add "Vacuum" dust --tag living-room

# List records
i-rs-filter list
```