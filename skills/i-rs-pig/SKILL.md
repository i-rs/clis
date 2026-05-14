---
name: "i-rs-pig"
description: "Records food cravings and junk food indulgences. Invoke when user wants to track moments of unhealthy eating or cravings."
---

# i-rs-pig

Craving and junk food tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/pig.json`

## Commands

### add

Record a craving or indulgence.

```bash
i-rs-pig add <FOOD_NAME> [OPTIONS]
```

Options:
- `--description <DESC>` - Description
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List craving records.

```bash
i-rs-pig list
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-pig get <ID>
```

### delete

Delete a record.

```bash
i-rs-pig delete <ID>
```

## Examples

```bash
# Record a craving
i-rs-pig add "Chocolate bar"
i-rs-pig add "French fries" --remark "Fast food lunch"

# List records
i-rs-pig list

# Get details
i-rs-pig get abc12345
```