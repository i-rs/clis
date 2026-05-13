# i-rs-remind Usage Guide

## Install

```bash
npm install -g @i-rs/i-rs-remind
# or
brew install i-rs/homebrew-tap/i-rs-remind
```

## Commands

### add

Add a new reminder.

```bash
i-rs-remind add <NAME> <EVENT_DATE> [OPTIONS]
```

Options:
- `-t, --title <TITLE>` - Event title
- `-T, --tag <TAG>` - Tags (can be used multiple times)
- `-c, --content <CONTENT>` - Content lines (can be used multiple times)

### list

List all reminders.

```bash
i-rs-remind list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get reminder details.

```bash
i-rs-remind get <NAME>
```

### done

Mark a reminder as done.

```bash
i-rs-remind done <NAME>
```

### update

Update a reminder.

```bash
i-rs-remind update <NAME> [OPTIONS]
```

Options:
- `-e, --event-date <DATE>` - New event date (YYYY-MM-DD or YYYY-MM-DD HH:MM)
- `-t, --title <TITLE>` - New title
- `-T, --tag <TAG>` - New tags
- `-c, --content <CONTENT>` - New content lines

### delete

Delete a reminder.

```bash
i-rs-remind delete <NAME>
```

## Examples

```bash
# Add a reminder
i-rs-remind add meeting 2025-06-15 14:00 --title "Team Meeting" --tag work --content "Discuss project"

# Add a birthday reminder
i-rs-remind add birthday 2025-08-20 --title "Friend's Birthday" --tag personal

# List all reminders
i-rs-remind list

# Get reminder details
i-rs-remind get meeting

# Mark as done
i-rs-remind done meeting

# Update reminder
i-rs-remind update meeting --content "New agenda"

# Delete a reminder
i-rs-remind delete meeting
```

## Data Storage

- macOS: `~/.config/i-rs/reminds.json`
- Linux: `~/.config/i-rs/reminds.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`
