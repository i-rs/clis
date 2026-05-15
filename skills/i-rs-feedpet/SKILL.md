---
name: "i-rs-feedpet"
description: "Records pet feeding. Invoke when user wants to track when they feed their pets."
---

# i-rs-feedpet

Pet feeding tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/feedpet.json`

## Global Flags

- `--json` — Output in JSON format
## Commands

### add

Record pet feeding.

```bash
i-rs-feedpet add <PET_NAME> <FOOD_TYPE> <AMOUNT>
```

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List feeding records.

```bash
i-rs-feedpet list
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-feedpet get <ID>
```

### delete

Delete a record.

```bash
i-rs-feedpet delete <ID>
```

### data

Manage data (export, import, clear).

```bash
i-rs-feedpet data export
i-rs-feedpet data import [FILE]
i-rs-feedpet data clear
```

### example

Show usage examples.

```bash
i-rs-feedpet example
```

### skill

Show skill information.

```bash
i-rs-feedpet skill [summary|content|raw]
```

## Examples

```bash
# Record feeding
i-rs-feedpet add "Cat" dry-food "50g"
i-rs-feedpet add "Dog" wet-food "200g" --tag morning

# List records
i-rs-feedpet list
```