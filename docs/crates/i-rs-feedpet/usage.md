# i-rs-feedpet Usage

## Commands

### add

Add a pet feeding record.

```bash
i-rs-feedpet add <PET_NAME> <FOOD_TYPE> <AMOUNT> [OPTIONS]
```

Arguments:
- `PET_NAME` - Name of the pet
- `FOOD_TYPE` - Type of food (dry-food, wet-food, treat, etc.)
- `AMOUNT` - Amount (e.g., "50g", "1 cup")

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List feeding records.

```bash
i-rs-feedpet list [OPTIONS]
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

## Food Types

| Type | Description |
|------|-------------|
| dry-food | Dry kibble |
| wet-food | Canned food |
| treat | Snacks/treats |
| raw | Raw food diet |

## Data Storage

- macOS: `~/.config/i-rs/feedpet.json`
- Linux: `~/.config/i-rs/feedpet.json`
- Windows: `~\AppData\Roaming\i-rs\feedpet.json`