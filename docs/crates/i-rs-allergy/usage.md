# i-rs-allergy Usage

## Global Flags

- `--json` — Output in JSON format

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

### data

Manage data (export, import, clear).

```bash
i-rs-allergy data export
i-rs-allergy data import [FILE]
i-rs-allergy data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data

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

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-allergy list
```
