---
name: "i-rs-run"
description: "Running record tracker. Invoke when user needs to record running activities, view statistics, or manage training plans."
---

# i-rs-run

Running record CLI tool for tracking running activities with detailed metrics.

## Storage

- Config: `~/.config/i-rs/runs.json`

## Global Flags

- `--json` — Output in JSON format
## Commands

### add
Add a running record.
```bash
i-rs-run add <DATE> <DISTANCE> <DURATION> [--heart-rate] [--weather] [--tags] [--remark]
```

### list
List all run records.
```bash
i-rs-run list [--json]
```

### get
Get run record details.
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
Add a running plan.
```bash
i-rs-run plan-add <NAME> <TARGET> <PACE> [--schedule] [--tags] [--remark]
```

### plan-list
List all running plans.
```bash
i-rs-run plan-list [--json]
```

### plan-get
Get plan details.
```bash
i-rs-run plan-get <ID> [--json]
```

### plan-delete
Delete a running plan.
```bash
i-rs-run plan-delete <ID>
```

### data

Manage data (export, import, clear).

```bash
i-rs-run data export
i-rs-run data import [FILE]
i-rs-run data clear
```

### example

Show usage examples.

```bash
i-rs-run example
```

### skill

Show skill information.

```bash
i-rs-run skill [summary|content|raw]
```

## Examples

```bash
# Add a run
i-rs-run add 2025-06-14 5.0 30 -r 145 -w sunny

# View stats
i-rs-run stats

# Create a plan
i-rs-run plan-add "5K Training" 5.0 6:00 --schedule 1 3 5
```
