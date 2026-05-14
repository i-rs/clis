---
name: "i-rs-dose"
description: "Records medicine dosage. Invoke when user wants to track when they take medications."
---

# i-rs-dose

Medicine dosage tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/dose.json`

## Commands

### add

Record medicine intake.

```bash
i-rs-dose add <MEDICINE_NAME> --dosage <AMOUNT> --unit <UNIT> [OPTIONS]
```

Options:
- `--dosage <AMOUNT>` - Dosage amount
- `--unit <UNIT>` - Unit (tablet, ml, mg, IU, etc.)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List medicine records.

```bash
i-rs-dose list
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-dose get <ID>
```

### delete

Delete a record.

```bash
i-rs-dose delete <ID>
```

## Examples

```bash
# Record medicine
i-rs-dose add "Vitamin D" --dosage 1000 --unit IU
i-rs-dose add "Ibuprofen" --dosage 400 --unit mg

# List records
i-rs-dose list

# Get details
i-rs-dose get abc12345
```