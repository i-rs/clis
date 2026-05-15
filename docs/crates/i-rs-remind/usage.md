# i-rs-remind Usage Guide

## Global Flags

- `--json` — Output in JSON format

## Commands Overview

| Command | Description |
|---------|-------------|
| `add` | Add a new reminder |
| `list` | List all reminders |
| `get` | Get reminder details |
| `done` | Mark reminder as done |
| `update` | Update a reminder |
| `delete` | Delete a reminder |

## add

Add a new reminder.

```bash
i-rs-remind add <NAME> <EVENT_DATE> [OPTIONS]
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `NAME` | Reminder name (unique identifier) | Yes |
| `EVENT_DATE` | Event date and optional time (YYYY-MM-DD or YYYY-MM-DD HH:MM) | Yes |

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-t` | `--title` | Event title |
| `-T` | `--tag` | Tags (can be specified multiple times) |
| `-c` | `--content` | Content lines (can be specified multiple times) |

### Examples

```bash
# Add reminder with date only
i-rs-remind add birthday 2025-08-20 --title "Friend's Birthday" --tag personal

# Add reminder with date and time
i-rs-remind add meeting 2025-06-15 14:00 --title "Team Meeting" --tag work --content "Discuss project"

# Add reminder with multiple tags
i-rs-remind add deadline 2025-07-01 --title "Project Deadline" --tag work --tag project --tag critical
```

---

## list

List all reminders, optionally filtered by tag.

```bash
i-rs-remind list [OPTIONS]
```

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-t` | `--tag` | Filter by tag |

---

## get

Get detailed information about a reminder.

```bash
i-rs-remind get <NAME>
```

---

## done

Mark a reminder as done (completed).

```bash
i-rs-remind done <NAME>
```

### Note

Marking a reminder as done marks it as completed but keeps it in the list. Use `delete` to remove it entirely.

---

## update

Update an existing reminder.

```bash
i-rs-remind update <NAME> [OPTIONS]
```

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-e` | `--event-date` | New event date (YYYY-MM-DD or YYYY-MM-DD HH:MM) |
| `-t` | `--title` | New title |
| `-T` | `--tag` | New tags |
| `-c` | `--content` | New content lines |

---

## delete

Delete a reminder.

```bash
i-rs-remind delete <NAME>
```

### data

Manage data (export, import, clear).

```bash
i-rs-remind data export
i-rs-remind data import [FILE]
i-rs-remind data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
### example

Show usage examples.

```bash
i-rs-remind example
```
### skill

Show skill information.

```bash
i-rs-remind skill [summary|content|raw]
```
