# i-rs-bed Usage

## Commands

### add

Add a bed item replacement record.

```bash
i-rs-bed add <ITEM_TYPE> [OPTIONS]
```

Arguments:
- `ITEM_TYPE` - Type of item (mattress, pillow, duvet, mattress-protector)

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List bed item replacement records.

```bash
i-rs-bed list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-bed get <ID>
```

### delete

Delete a record.

```bash
i-rs-bed delete <ID>
```

## Bed Item Types

| Type | Description | Replacement Interval |
|------|-------------|---------------------|
| mattress | Mattress | 7-10 years |
| pillow | Pillow | 1-2 years |
| duvet | Duvet/comforter | 5-10 years |
| mattress-protector | Mattress protector | 2-3 years |

## Data Storage

- macOS: `~/.config/i-rs/beds.json`
- Linux: `~/.config/i-rs/beds.json`
- Windows: `~\AppData\Roaming\i-rs\bed.json`