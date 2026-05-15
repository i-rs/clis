---
name: "i-rs-time"
description: "Time tracking CLI for work hours (Pomodoro timer). Invoke when tracking work time, starting/stopping timers, viewing statistics, or generating work reports."
---

# i-rs-time

Time tracking CLI for work hours (Pomodoro timer).

## Storage

- Config: `~/.config/i-rs/time.json`

## Global Flags

- `--json` — Output in JSON format
## Commands

### start

Start a new timer.

```bash
i-rs-time start "Task name"
i-rs-time start "Meeting" --tag work
i-rs-time start "Coding" --tag development --remark "Feature implementation"
```

### stop

Stop the current timer.

```bash
i-rs-time stop
```

### list

List all time entries.

```bash
i-rs-time list [OPTIONS]
i-rs-time list --tag work
```

### stats

Show statistics.

```bash
i-rs-time stats today
i-rs-time stats yesterday
i-rs-time stats week
```

### report

Generate work reports.

```bash
i-rs-time report --days 7
i-rs-time report --start 2024-01-01 --end 2024-01-31
```

### get

Get entry details.

```bash
i-rs-time get <entry-id>
```

### delete

Delete an entry.

```bash
i-rs-time delete <entry-id>
```

### example

Show usage examples.

```bash
i-rs-time example
```

### skill

View AI skill documentation.

```bash
i-rs-time skill
i-rs-time skill summary
```

### data

Manage data (export, import, clear).

```bash
i-rs-time data export
i-rs-time data import [FILE]
i-rs-time data clear
```

## Examples

```bash
# Start tracking
i-rs-time start "Working on project" --tag work

# Stop when done
i-rs-time stop

# View daily stats
i-rs-time stats today

# Weekly report
i-rs-time report --days 7

# List all entries
i-rs-time list [OPTIONS]

# Filter by tag
i-rs-time list --tag work
```
