---
name: "i-rs-habit"
description: "Tracks habits with checkins and streaks. Invoke when user wants to create, track, or checkin habits."
---

# i-rs-habit

Habit tracking CLI tool for building good habits with checkins and streaks.

## Storage

- Config: `~/.config/i-rs/habits.json`

## Commands

### add

Create a new habit.

```bash
i-rs-habit add <NAME> [OPTIONS]
```

Options:
- `-d, --description <TEXT>` - Habit description
- `-f, --frequency <FREQ>` - Frequency (daily/weekly/monthly/yearly, default: daily)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### checkin

Checkin for a habit.

```bash
i-rs-habit checkin <NAME>
```

### list

List all habits.

```bash
i-rs-habit list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get habit details.

```bash
i-rs-habit get <NAME>
```

### update

Update a habit.

```bash
i-rs-habit update <NAME> [OPTIONS]
```

### delete

Delete a habit.

```bash
i-rs-habit delete <NAME>
```

## Examples

```bash
# Create a daily habit
i-rs-habit add daily_walk --description "Walk 30 minutes" --frequency daily --tag health

# Checkin for today
i-rs-habit checkin daily_walk

# List habits with streaks
i-rs-habit list
```

## Streak Calculation

Streaks are calculated based on consecutive checkins. A streak breaks if there's more than one day gap between checkins.