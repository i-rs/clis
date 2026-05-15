---
name: "i-rs-allergy"
description: "Records allergy reactions. Invoke when user wants to track allergic reactions and symptoms."
---

# i-rs-allergy

Allergy tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/allergies.json`

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Record allergy reaction.

```bash
i-rs-allergy add <ALLERGEN> <SEVERITY> [OPTIONS]
```

Options:
- `-s, --symptom <SYMPTOM>` - Symptoms (can be repeated)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List allergy records.

```bash
i-rs-allergy list [OPTIONS]
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

### data

Manage data (export, import, clear).

```bash
i-rs-allergy data export
i-rs-allergy data import [FILE]
i-rs-allergy data clear
```

### example

Show usage examples.

```bash
i-rs-allergy example
```

### skill

Show skill information.

```bash
i-rs-allergy skill [summary|content|raw]
```

## Severity Levels

- `mild` - Minor reaction
- `moderate` - Noticeable reaction
- `severe` - Serious reaction

## Examples

```bash
# Record allergy
i-rs-allergy add "Peanuts" mild --symptom hives [OPTIONS]
i-rs-allergy add "Pollen" severe --symptom sneezing [OPTIONS]

# List records
i-rs-allergy list [OPTIONS]
```
