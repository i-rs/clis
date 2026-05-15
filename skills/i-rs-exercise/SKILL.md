---
name: "i-rs-exercise"
description: "Exercise record tracking CLI tool for logging and managing fitness activities. Invoke when user needs to track workouts, log exercises, view exercise history, show statistics, or manage fitness data."
---

# i-rs-exercise

## Global Flags

- `--json` — Output in JSON format

Exercise record tracking CLI tool for logging and managing fitness activities.

## Storage

- Config: `~/.config/i-rs/exercises.json`

## Commands

### add

Add a new exercise record.

```bash
i-rs-exercise add <NAME> <TYPE> <DURATION>
```

Arguments:
- `NAME` - Exercise name
- `TYPE` - Exercise type (e.g. running, swimming, gym, yoga, cycling, hiit)
- `DURATION` - Duration in minutes

Options:
- `-c, --calories <CALORIES>` - Calories burned
- `-t, --tag <TAG>` - Tags (repeatable)
- `-n, --notes <NOTES>` - Notes (repeatable)
- `-r, --remark <REMARK>` - Remarks (repeatable)

### list

List exercise records.

```bash
i-rs-exercise list
```

Options:
- `-t, --tag <TAG>` - Filter by tag
- `-y, --exercise-type <TYPE>` - Filter by exercise type

### get

Get exercise record details.

```bash
i-rs-exercise get <NAME>
```

### update

Update an exercise record.

```bash
i-rs-exercise update <NAME> [OPTIONS]
```

Options:
- `-y, --exercise-type <TYPE>` - New exercise type
- `-d, --duration-minutes <DURATION>` - New duration
- `-c, --calories <CALORIES>` - New calories (use `--calories ''` to clear)
- `-t, --tag <TAG>` - New tags
- `-n, --notes <NOTES>` - New notes
- `-r, --remark <REMARK>` - New remarks

### delete

Delete an exercise record.

```bash
i-rs-exercise delete <NAME>
```

### stats

Show exercise statistics.

```bash
i-rs-exercise stats
```

Shows:
- Total records
- Total exercise duration
- Total calories burned
- Exercise type count
- Most common exercise type
- Longest duration type
- Detailed breakdown by type

### example

Show usage examples.

```bash
i-rs-exercise example
```

### skill

View AI skill documentation.

```bash
i-rs-exercise skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/exercises.json`
- Linux: `~/.config/i-rs/exercises.json`
- Windows: `~\AppData\Roaming\i-rs\exercises.json`

## JSON Output

All commands support `--json` global flag:

```bash
i-rs-exercise list --json
i-rs-exercise stats --json
i-rs-exercise get "Morning Run" --json
```
