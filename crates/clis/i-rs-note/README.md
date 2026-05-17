# i-rs-note

Note management CLI tool for managing notes locally.

## Install

```bash
npm install -g @i-rs/i-rs-note
# or
brew install i-rs/homebrew-tap/i-rs-note
```

## Usage

### Add Note

```bash
i-rs-note add <NAME> [OPTIONS]
```

Options:
- `-t, --title <TITLE>` - Note title
- `-T, --tag <TAG>` - Tags (can be specified multiple times)
- `-c, --content <CONTENT>` - Content lines (can be specified multiple times)

### List Notes

```bash
i-rs-note list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### Get Note Details

```bash
i-rs-note get <NAME>
```

### Update Note

```bash
i-rs-note update <NAME> [OPTIONS]
```

Options:
- `-t, --title <TITLE>` - New title
- `-T, --tag <TAG>` - New tags
- `-c, --content <CONTENT>` - New content lines

### Delete Note

```bash
i-rs-note delete <NAME>
```

## Examples

```bash
# Add a simple note
i-rs-note add todo --title "Todo List" --tag work --content "Buy groceries" --content "Call mom"

# Add a note with multiple tags
i-rs-note add ideas --title "Project Ideas" --tag project --tag brainstorm --content "Idea 1" --content "Idea 2"

# List all notes
i-rs-note list

# List notes by tag
i-rs-note list --tag work

# Get note details
i-rs-note get todo

# Update note
i-rs-note update todo --content "New content line"

# Delete a note
i-rs-note delete todo
```

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/notes.json`
- Linux: `~/.config/i-rs/notes.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-note list
```

## License

MIT OR Apache-2.0
