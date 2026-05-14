---
name: "i-rs-allergy"
description: "Records allergy reactions. Invoke when user wants to track allergic reactions and symptoms."
---

# i-rs-allergy

Allergy tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/allergies.json`

## Commands

### add

Record allergy reaction.

```bash
i-rs-allergy add <ALLERGEN> <SEVERITY>
```

Options:
- `-s, --symptom <SYMPTOM>` - Symptoms (can be repeated)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List allergy records.

```bash
i-rs-allergy list
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-allergy get <ID>
```

### delete

Delete a record.

```bash
i-rs-allergy delete <ID>
```

## Severity Levels

- `mild` - Minor reaction
- `moderate` - Noticeable reaction
- `severe` - Serious reaction

## Examples

```bash
# Record allergy
i-rs-allergy add "Peanuts" mild --symptom hives
i-rs-allergy add "Pollen" severe --symptom sneezing

# List records
i-rs-allergy list
```