# i-rs-remind

Reminder management CLI tool for managing events and reminders.

## Install

```bash
npm install -g @i-rs/i-rs-remind
# or
brew install i-rs/homebrew-tap/i-rs-remind
```

## Usage

### Add Reminder

```bash
i-rs-remind add <NAME> <EVENT_DATE> [OPTIONS]
```

Options:
- `-t, --title <TITLE>` - Event title
- `-T, --tag <TAG>` - Tags (can be specified multiple times)
- `-c, --content <CONTENT>` - Content lines (can be specified multiple times)

### List Reminders

```bash
i-rs-remind list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### Get Reminder Details

```bash
i-rs-remind get <NAME>
```

### Mark as Done

```bash
i-rs-remind done <NAME>
```

### Update Reminder

```bash
i-rs-remind update <NAME> [OPTIONS]
```

Options:
- `-e, --event-date <DATE>` - New event date (YYYY-MM-DD or YYYY-MM-DD HH:MM)
- `-t, --title <TITLE>` - New title
- `-T, --tag <TAG>` - New tags
- `-c, --content <CONTENT>` - New content lines

### Delete Reminder

```bash
i-rs-remind delete <NAME>
```

## Examples

```bash
# Add a reminder
i-rs-remind add meeting 2025-06-15 14:00 --title "Team Meeting" --tag work --content "Discuss project进展"

# Add a birthday reminder
i-rs-remind add birthday 2025-08-20 --title "Friend's Birthday" --tag personal

# List all reminders
i-rs-remind list

# List reminders by tag
i-rs-remind list --tag work

# Get reminder details
i-rs-remind get meeting

# Mark as done
i-rs-remind done meeting

# Update reminder
i-rs-remind update meeting --content "New agenda items"

# Delete a reminder
i-rs-remind delete meeting
```

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/reminds.json`
- Linux: `~/.config/i-rs/reminds.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-remind list
```

## License

MIT OR Apache-2.0
