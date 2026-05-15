# i-rs-habit Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Create a new habit.

```bash
i-rs-habit add <NAME> [OPTIONS]
```

Arguments:
- `NAME` - Habit name (required)

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

Arguments:
- `NAME` - Habit name (required)

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

Arguments:
- `NAME` - Habit name (required)

### update

Update a habit.

```bash
i-rs-habit update <NAME> [OPTIONS]
```

Options:
- `-d, --description <TEXT>` - Update description
- `-f, --frequency <FREQ>` - Update frequency
- `-t, --tag <TAG>` - Update tags
- `-r, --remark <REMARK>` - Update remarks

### delete

Delete a habit.

```bash
i-rs-habit delete <NAME>
```

Arguments:
- `NAME` - Habit name (required)

## Frequency Options

| Option | Description |
|--------|-------------|
| daily | Every day |
| weekly | Once a week |
| monthly | Once a month |
| yearly | Once a year |
| custom | Custom frequency |

### data

Manage data (export, import, clear).

```bash
i-rs-habit data export
i-rs-habit data import [FILE]
i-rs-habit data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
### example

Show usage examples.

```bash
i-rs-habit example
```
### skill

Show skill information.

```bash
i-rs-habit skill [summary|content|raw]
```

## Data Storage

- macOS: `~/Library/Application Support/i-rs/habits.json`
- Linux: `~/.config/i-rs/habits.json`
- Windows: `~\AppData\Roaming\i-rs\habits.json`