# i-rs-mood Usage

## Global Flags

- `--json` — Output in JSON format

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
i-rs-mood add <MOOD> [OPTIONS]
```

Arguments:
- `MOOD` - Mood level (see Mood Levels below)

Options:
- `-D, --date <DATE>` - Date in YYYY-MM-DD format (default: today)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List mood records.

```bash
i-rs-mood list [OPTIONS]
```

Options:
- `-d, --days <DAYS>` - Show records from last N days
- `-c, --calendar` - Show mood calendar

### get

Get a mood record by id.

```bash
i-rs-mood get <ID>
```

### update

Update a mood record by id.

```bash
i-rs-mood update <ID> [OPTIONS]
```

Options:
- `-D, --date <DATE>` - New date
- `-m, --mood <MOOD>` - New mood level
- `-t, --tag <TAG>` - New tags
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### delete

Delete a mood record by id.

```bash
i-rs-mood delete <ID>
```

## Mood Levels

| Number | Word | Emoji | Description |
|--------|------|-------|-------------|
| 7 | amazing | 🤩 | Excellent mood |
| 6 | great | 😊 | Great mood |
| 5 | good | 🙂 | Positive mood |
| 4 | okay/ok | 😐 | Neutral mood |
| 3 | poor | 😕 | Low mood |
| 2 | bad | 😔 | Negative mood |
| 1 | terrible | 😢 | Very negative mood |

All input formats are accepted:
- Numbers: `1`, `2`, `3`, `4`, `5`, `6`, `7`
- Words: `amazing`, `great`, `good`, `okay`, `poor`, `bad`, `terrible`
- Emoji: `🤩`, `😊`, `🙂`, `😐`, `😕`, `😔`, `😢`

## Examples

### Basic Usage

```bash
# Record today's mood
i-rs-mood add good

# Record with emoji for a specific date
i-rs-mood add 😊 --date 2025-01-16

# Record with number
i-rs-mood add 4 --date 2025-01-17
```

### With Tags

```bash
# Single tag
i-rs-mood add good --tag work --date 2025-01-18

# Multiple tags
i-rs-mood add great --tag weekend --tag family --tag exercise --date 2025-01-19
```

### With Remarks

```bash
# Single remark
i-rs-mood add okay --remark "Monday blues" --date 2025-01-20

# Multiple remarks
i-rs-mood add good --remark "Project completed" --remark "Feeling accomplished" --date 2025-01-21
```

### data

Manage data (export, import, clear).

```bash
i-rs-mood data export
i-rs-mood data import [FILE]
i-rs-mood data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
### example

Show usage examples.

```bash
i-rs-mood example
```
### skill

Show skill information.

```bash
i-rs-mood skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/mood.json`
- Linux: `~/.config/i-rs/mood.json`
- Windows: `~\AppData\Roaming\i-rs\mood.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-mood list
```
