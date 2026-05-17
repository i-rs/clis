# i-rs-exercise

Exercise record tracking CLI tool for logging and managing fitness activities.

## Features

- **Record Management**: Add, view, update, delete exercise records
- **Multiple Types**: Support custom exercise types (running, swimming, gym, etc.)
- **Calorie Tracking**: Optional calorie tracking per session
- **Tags**: Organize and filter records with tags
- **Statistics**: View total duration, calories burned, and more
- **JSON Output**: All commands support `--json` global flag

## Install

```bash
npm install -g @i-rs/i-rs-exercise
# or
brew install i-rs/homebrew-tap/i-rs-exercise
```

## Quick Start

```bash
# Add an exercise record
i-rs-exercise add "Morning Run" running 30 -c 300 -t morning -t cardio

# List all records
i-rs-exercise list

# Filter by tag
i-rs-exercise list --tag cardio

# Filter by exercise type
i-rs-exercise list --exercise-type running

# View statistics
i-rs-exercise stats

# Get record details
i-rs-exercise get "Morning Run"

# Update record
i-rs-exercise update "Morning Run" --duration-minutes 45

# Delete record
i-rs-exercise delete "Morning Run"
```

## Commands

### add

Add a new exercise record.

```bash
i-rs-exercise add <NAME> <TYPE> <DURATION> [OPTIONS]
```

Arguments:
- `NAME` - Exercise name
- `TYPE` - Exercise type (e.g. running, swimming, gym)
- `DURATION` - Duration in minutes

Options:
- `-c, --calories <CALORIES>` - Calories burned
- `-t, --tag <TAG>` - Tags (repeatable)
- `-n, --notes <NOTES>` - Notes (repeatable)
- `-r, --remark <REMARK>` - Remarks (repeatable)

### list

List all exercise records.

```bash
i-rs-exercise list [OPTIONS]
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

Update an existing exercise record.

```bash
i-rs-exercise update <NAME> [OPTIONS]
```

Options:
- `-y, --exercise-type <TYPE>` - New exercise type
- `-d, --duration-minutes <DURATION>` - New duration
- `-c, --calories <CALORIES>` - New calories (use `--calories ''` to clear)
- `-t, --tag <TAG>` - New tags (overwrites)
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

Override with `CONFIG_DIR` environment variable.

## JSON Output

All commands support `--json` global flag:

```bash
i-rs-exercise list --json
i-rs-exercise stats --json
i-rs-exercise get "Morning Run" --json
```

## License

MIT OR Apache-2.0
