---
name: "i-rs-cycle"
description: "Records menstrual cycle events. Invoke when user wants to track their period or related symptoms."
---

# i-rs-cycle

Menstrual cycle tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/cycles.json`

## Commands

### add

Record cycle event.

```bash
i-rs-cycle add <DATE> <EVENT_TYPE>
```

Options:
- `-s, --symptom <SYMPTOM>` - Symptoms (can be repeated)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List cycle records.

```bash
i-rs-cycle list
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

- `period` - Menstrual period
- `spotting` - Light bleeding
- `ovulation` - Ovulation day
- `fertile` - Fertile window

## Examples

```bash
# Record period
i-rs-cycle add 2024-01-15 period

# Record with symptoms
i-rs-cycle add 2024-01-15 period --symptom cramps

# List records
i-rs-cycle list
```