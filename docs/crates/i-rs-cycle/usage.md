# i-rs-cycle Usage

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

## Data Storage

- macOS: `~/.config/i-rs/cycles.json`
- Linux: `~/.config/i-rs/cycles.json`
- Windows: `~\AppData\Roaming\i-rs\cycle.json`