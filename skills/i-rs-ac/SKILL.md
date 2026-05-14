---
name: "i-rs-ac"
description: "Records AC cleaning. Invoke when user wants to track when they clean air conditioners."
---

# i-rs-ac

AC cleaning tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/ac.json`

## Commands

### add

Record AC cleaning.

```bash
i-rs-ac add <LOCATION>
```

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List AC cleaning records.

```bash
i-rs-ac list
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-ac get <ID>
```

### delete

Delete a record.

```bash
i-rs-ac delete <ID>
```

## Examples

```bash
# Record cleaning
i-rs-ac add "Living Room"
i-rs-ac add "Bedroom" --tag summer

# List records
i-rs-ac list
```