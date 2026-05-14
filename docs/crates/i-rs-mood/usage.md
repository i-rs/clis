# i-rs-mood Usage

## install

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

| Number | Word | Emoji | Description |
|--------|------|-------|-------------|
| 5 | great | 😊 | Excellent mood |
| 4 | good | 🙂 | Positive mood |
| 3 | okay/ok | 😐 | Neutral mood |
| 2 | bad | 😔 | Negative mood |
| 1 | terrible | 😢 | Very negative mood |

All input formats are accepted:
- Numbers: `1`, `2`, `3`, `4`, `5`
- Words: `great`, `good`, `okay`, `bad`, `terrible`
- Emoji: `😊`, `🙂`, `😐`, `😔`, `😢`

## Examples

### Basic Usage

```bash
# Record today's mood (assuming today is 2025-01-15)
i-rs-mood add 2025-01-15 good

# Record with emoji
i-rs-mood add 2025-01-16 😊

# Record with number
i-rs-mood add 2025-01-17 3
```

### With Tags

```bash
# Single tag
i-rs-mood add 2025-01-18 good --tag work

# Multiple tags
i-rs-mood add 2025-01-19 great --tag weekend --tag family --tag exercise
```

### With Notes

```bash
# Single note
i-rs-mood add 2025-01-20 okay --content "Monday blues"

# Multiple notes
i-rs-mood add 2025-01-21 good --content "Project completed" --content "Feeling accomplished"
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
