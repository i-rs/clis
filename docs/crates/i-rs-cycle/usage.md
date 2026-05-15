# i-rs-cycle Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Add a cycle event record.

```bash
i-rs-cycle add <DATE> <EVENT_TYPE> [OPTIONS]
```

Arguments:
- `DATE` - Date in YYYY-MM-DD format
- `EVENT_TYPE` - Type of event (period, spotting, ovulation, fertile)

Options:
- `-s, --symptom <SYMPTOM>` - Symptoms (can be repeated)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List cycle records.

```bash
i-rs-cycle list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-cycle get <ID>
```

### delete

Delete a record.

```bash
i-rs-cycle delete <ID>
```

## Event Types

| Type | Description |
|------|-------------|
| period | Menstrual period |
| spotting | Light bleeding |
| ovulation | Ovulation day |
| fertile | Fertile window |

## Common Symptoms

- cramps
- headache
- bloating
- mood-swings
- fatigue
- breast-tenderness

### data

Manage data (export, import, clear).

```bash
i-rs-cycle data export
i-rs-cycle data import [FILE]
i-rs-cycle data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
### example

Show usage examples.

```bash
i-rs-cycle example
```
### skill

Show skill information.

```bash
i-rs-cycle skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/cycles.json`
- Linux: `~/.config/i-rs/cycles.json`
- Windows: `~\AppData\Roaming\i-rs\cycle.json`