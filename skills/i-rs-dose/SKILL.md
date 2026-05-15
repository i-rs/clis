---
name: "i-rs-dose"
description: "Records medicine dosage. Invoke when user wants to track when they take medications."
---

# i-rs-dose

Medicine dosage tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/dose.json`

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Record medicine intake.

```bash
i-rs-dose add <MEDICINE_NAME> <DOSAGE> <UNIT> [OPTIONS]
```

Arguments:
- `MEDICINE_NAME` - Name of the medicine
- `DOSAGE` - Dosage amount
- `UNIT` - Unit (tablet, ml, mg, IU, etc.)

Options:
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

### data

Manage data (export, import, clear).

```bash
i-rs-dose data export
i-rs-dose data import [FILE]
i-rs-dose data clear
```

### example

Show usage examples.

```bash
i-rs-dose example
```

### skill

Show skill information.

```bash
i-rs-dose skill [summary|content|raw]
```

## Examples

```bash
# Record medicine
i-rs-dose add "Vitamin D" 1000 IU
i-rs-dose add "Ibuprofen" 400 mg

# List records
i-rs-dose list

# Get details
i-rs-dose get abc12345

# JSON output
i-rs-dose list --json
```
