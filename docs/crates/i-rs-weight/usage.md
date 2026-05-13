# i-rs-weight Usage Guide

## Commands Overview

| Command | Description |
|---------|-------------|
| `add` | Add a weight record |
| `list` | List weight records with optional chart and stats |
| `update` | Update a weight record |
| `delete` | Delete a weight record |

## add

Add a new weight record.

```bash
i-rs-weight add <DATE> <WEIGHT> [OPTIONS]
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `DATE` | Date (YYYY-MM-DD) | Yes |
| `WEIGHT` | Weight value (kg) | Yes |

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-r` | `--remark` | Remarks (can be specified multiple times) |

### Examples

```bash
# Add record for today
i-rs-weight add 2025-01-15 70.5

# Add with remark
i-rs-weight add 2025-01-16 70.3 --remark "After workout"

# Add multiple remarks
i-rs-weight add 2025-01-17 70.1 --remark "Morning weight" --remark "Fasted"
```

---

## list

List weight records with optional time range, chart, and statistics.

```bash
i-rs-weight list [OPTIONS]
```

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-d` | `--days` | Show records from last N days |
| `-c` | `--chart` | Show ASCII trend chart |
| `-s` | `--stats` | Show statistics (min/max/avg/change) |

### Chart Output Example

```
Weight Trend (Last 30 days)
─────────────────────────────────────
 71.0 ●
 70.5    │
 70.0 ●──│──●
 69.5    │
 69.0 ●────────────────────●
 68.5
 68.0

  01-15               01-20

  70.5 → 68.0 (-2.5 kg ↓)
```

### Statistics Output

Shows:
- **Min**: Minimum weight in range
- **Max**: Maximum weight in range
- **Avg**: Average weight
- **Change**: Total change (start → end) with trend arrow

---

## update

Update an existing weight record.

```bash
i-rs-weight update <DATE> [OPTIONS]
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `DATE` | Date of record to update | Yes |

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-w` | `--weight` | New weight value |
| `-r` | `--remark` | New remarks (replaces all) |

### Examples

```bash
# Update weight value
i-rs-weight update 2025-01-15 --weight 70.0

# Update remarks
i-rs-weight update 2025-01-15 --remark "Corrected measurement"

# Update both
i-rs-weight update 2025-01-15 -w 69.5 --remark "Morning weight"
```

---

## delete

Delete a weight record.

```bash
i-rs-weight delete <DATE>
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `DATE` | Date of record to delete | Yes |

### Examples

```bash
# Delete a specific record
i-rs-weight delete 2025-01-15
```