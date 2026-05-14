---
name: "i-rs-feedpet"
description: "Records pet feeding. Invoke when user wants to track when they feed their pets."
---

# i-rs-feedpet

Pet feeding tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/feedpet.json`

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

## Examples

```bash
# Record feeding
i-rs-feedpet add "Cat" dry-food "50g"
i-rs-feedpet add "Dog" wet-food "200g" --tag morning

# List records
i-rs-feedpet list
```