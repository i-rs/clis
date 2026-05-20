# i-rs-weight Usage Guide

## Global Flags

- `--json` — Output in JSON format

## Commands

| Command | Description |
|---------|-------------|
| `add` | Add a weight record |
| `get` | Get a weight record by ID |
| `list` | List weight records with optional chart and stats |
| `update` | Update a weight record by ID |
| `delete` | Delete a weight record by ID |

## add

Add a new weight record.

```bash
i-rs-weight add <WEIGHT> [OPTIONS]
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `WEIGHT` | Weight value (kg) | Yes |

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-d` | `--date` | Date (YYYY-MM-DD, defaults to today) |
| `-t` | `--tag` | Tags (can be repeated) |
| `-r` | `--remark` | Remarks (can be repeated) |

### Examples

```bash
# Add record for today
i-rs-weight add 70.5

# Add with date and remark
i-rs-weight add 70.3 --date 2025-01-16 --remark "After workout"

# Add with tags and remarks
i-rs-weight add 70.1 --date 2025-01-17 --tag morning --remark "Fasted"
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

## get

Get a weight record by ID.

```bash
i-rs-weight get <ID>
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `ID` | Record ID (supports short prefix) | Yes |

### Examples

```bash
# Get record by ID
i-rs-weight get abc12345
```

---

## update

Update an existing weight record.

```bash
i-rs-weight update <ID> [OPTIONS]
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `ID` | ID of record to update | Yes |

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-w` | `--weight` | New weight value |
| `-t` | `--tag` | New tags (replaces all) |
| `-r` | `--remark` | New remarks (replaces all) |

### Examples

```bash
# Update weight value
i-rs-weight update abc12345 --weight 70.0

# Update tags and remarks
i-rs-weight update abc12345 --tag morning --remark "Corrected measurement"
```

---

## delete

Delete a weight record.

```bash
i-rs-weight delete <ID>
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `ID` | ID of record to delete | Yes |

### Examples

```bash
# Delete a specific record
i-rs-weight delete abc12345
```

### data

Manage data (export, import, clear).

```bash
i-rs-weight data export
i-rs-weight data import [FILE]
i-rs-weight data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data

### example

Show usage examples.

```bash
i-rs-weight example
```

### skill

Show skill information.

```bash
i-rs-weight skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/weights.json`
- Linux: `~/.config/i-rs/weights.json`
- Windows: `~\AppData\Roaming\i-rs\weights.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-weight list
```
