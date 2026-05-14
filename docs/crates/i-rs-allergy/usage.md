# i-rs-allergy Usage

## Commands

### add

Add an allergy record.

```bash
i-rs-allergy add <ALLERGEN> <SEVERITY> [OPTIONS]
```

Arguments:
- `ALLERGEN` - The allergen (e.g., Peanuts, Pollen)
- `SEVERITY` - Severity level (mild, moderate, severe)

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

## Severity Levels

| Level | Description |
|-------|-------------|
| mild | Minor reaction, may not need treatment |
| moderate | Noticeable reaction, may need medication |
| severe | Serious reaction, may need medical attention |

## Common Symptoms

- hives
- itching
- swelling
- sneezing
- watery-eyes
- headache
- nausea

## Data Storage

- macOS: `~/.config/i-rs/allergies.json`
- Linux: `~/.config/i-rs/allergies.json`
- Windows: `~\AppData\Roaming\i-rs\allergy.json`