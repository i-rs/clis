# Usage

## Commands

### add

Add a new run record.

```bash
i-rs-run add <DATE> <DISTANCE> <DURATION> [OPTIONS]
```

**Arguments:**
- `DATE` - Date in YYYY-MM-DD format
- `DISTANCE` - Distance in kilometers
- `DURATION` - Duration in minutes

**Options:**
- `-r, --heart-rate <HR>` - Heart rate in bpm
- `-w, --weather <WEATHER>` - Weather conditions
- `-t, --tags <TAGS>` - Tags (repeatable)
- `--remark <REMARK>` - Remarks (repeatable)

**Example:**
```bash
i-rs-run add 2025-06-14 5.0 30 -r 145 -w sunny -t marathon
```

### list

List all run records.

```bash
i-rs-run list [--json]
```

### get

Get details of a specific run record.

```bash
i-rs-run get <ID> [--json]
```

### delete

Delete a run record.

```bash
i-rs-run delete <ID>
```

### stats

Show cumulative statistics.

```bash
i-rs-run stats
```

### plan-add

Add a new running plan.

```bash
i-rs-run plan-add <NAME> <TARGET> <PACE> [OPTIONS]
```

**Arguments:**
- `NAME` - Plan name
- `TARGET` - Target distance in km
- `PACE` - Target pace (e.g., 6:00 for 6 min/km)

**Options:**
- `-s, --schedule <DAYS>` - Schedule days (0-6 for Sun-Sat)
- `-t, --tags <TAGS>` - Tags
- `--remark <REMARK>` - Remarks

**Example:**
```bash
i-rs-run plan-add "5K Training" 5.0 6:00 --schedule 1 3 5
```

### plan-list

List all running plans.

```bash
i-rs-run plan-list [--json]
```

### plan-get

Get details of a specific plan.

```bash
i-rs-run plan-get <ID> [--json]
```

### plan-delete

Delete a running plan.

```bash
i-rs-run plan-delete <ID>
```

## Global Options

- `--json` - Output in JSON format
