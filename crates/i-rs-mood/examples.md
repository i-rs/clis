# i-rs-mood Usage Guide

## Install

```bash
npm install -g @i-rs/i-rs-mood
# or
brew install i-rs/homebrew-tap/i-rs-mood
```

## Commands

### add

Add a mood record.

```bash
i-rs-mood add <DATE> <MOOD> [OPTIONS]
```

Arguments:
- `DATE` - Date in YYYY-MM-DD format
- `MOOD` - Mood level (see Mood Levels below)

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-c, --content <CONTENT>` - Content/notes (can be repeated)

### list

List mood records.

```bash
i-rs-mood list [OPTIONS]
```

Options:
- `-d, --days <DAYS>` - Show records from last N days
- `-c, --calendar` - Show mood calendar

### update

Update a mood record.

```bash
i-rs-mood update <DATE> [OPTIONS]
```

Options:
- `-m, --mood <MOOD>` - New mood level
- `-t, --tag <TAG>` - New tags
- `-c, --content <CONTENT>` - New content

### delete

Delete a mood record.

```bash
i-rs-mood delete <DATE>
```

## Mood Levels

| Input | Emoji | Label |
|-------|-------|-------|
| 5, great, 😊 | 😊 | Great |
| 4, good, 🙂 | 🙂 | Good |
| 3, okay, ok, 😐 | 😐 | Okay |
| 2, bad, 😔 | 😔 | Bad |
| 1, terrible, 😢 | 😢 | Terrible |

## Examples

```bash
# Add a mood record
i-rs-mood add 2025-01-15 good

# Add with tags and notes
i-rs-mood add 2025-01-16 great --tag work --tag exercise --content "Feeling energetic after workout"

# Add using emoji
i-rs-mood add 2025-01-17 😊 --tag weekend --content "Great day with family"

# Add using number
i-rs-mood add 2025-01-18 3 --tag monday --content "Monday blues"

# List all records
i-rs-mood list

# List last 7 days with calendar
i-rs-mood list --days 7 --calendar

# List last 30 days
i-rs-mood list --days 30

# Update mood for a day
i-rs-mood update 2025-01-15 --mood okay --content "Actually feeling a bit tired"

# Delete a record
i-rs-mood delete 2025-01-15
```

## Data Storage

- macOS: `~/.config/i-rs/moods.json`
- Linux: `~/.config/i-rs/moods.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-mood list
```
