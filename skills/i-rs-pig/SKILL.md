---
name: "i-rs-pig"
description: "Records food cravings and junk food indulgences. Invoke when user wants to track moments of unhealthy eating or cravings."
---

# i-rs-pig

Craving and junk food tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/pig.json`

## Global Flags

- `--json` — Output in JSON format
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
i-rs-pig list [OPTIONS]
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

### update

Update a craving record.

```bash
i-rs-pig update <ID> [OPTIONS]
```

Options:
- `--food-name <NAME>` - Food name
- `--description <DESC>` - Description
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### data

Manage data (export, import, clear).

```bash
i-rs-pig data export
i-rs-pig data import [FILE]
i-rs-pig data clear
```

### example

Show usage examples.

```bash
i-rs-pig example
```

### skill

Show skill information.

```bash
i-rs-pig skill [summary|content|raw]
```

## Examples

```bash
# Record a craving
i-rs-pig add "Chocolate bar" [OPTIONS]
i-rs-pig add "French fries" --remark "Fast food lunch" [OPTIONS]

# List records
i-rs-pig list [OPTIONS]

# Get details
i-rs-pig get abc12345
```