---
name: "i-rs-remind"
description: "Manages reminders (add/list/get/update/delete/done). Invoke when user needs to track events, deadlines, birthdays, or schedule reminders."
---

# i-rs-remind

Reminder management CLI tool for managing events and reminders.

## Storage

- Config: `~/.config/i-rs/reminds.json`

## Commands

### add

Add a new reminder.

```bash
i-rs-remind add <NAME> <EVENT_DATE>
```

Options:
- `-t, --title <TITLE>` - Event title
- `-T, --tag <TAG>` - Tags (can be repeated)
- `-c, --content <CONTENT>` - Content lines (can be repeated)

### list

List reminders.

```bash
i-rs-remind list
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

Update reminder.

```bash
i-rs-remind update <NAME>
```

Options:
- `-e, --event-date <DATE>` - New event date (YYYY-MM-DD or YYYY-MM-DD HH:MM)
- `-t, --title <TITLE>` - New title
- `-T, --tag <TAG>` - New tags
- `-c, --content <CONTENT>` - New content lines

### delete

Delete reminder.

```bash
i-rs-remind delete <NAME>
```

## Examples

```bash
# Add a reminder
i-rs-remind add meeting 2025-06-15 14:00 --title "Team Meeting" --tag work

# Add a birthday
i-rs-remind add birthday 2025-08-20 --tag personal

# List all reminders
i-rs-remind list

# Get reminder details
i-rs-remind get meeting

# Mark as done
i-rs-remind done meeting

# Update reminder
i-rs-remind update meeting --content "New agenda"

# Delete reminder
i-rs-remind delete meeting
```
