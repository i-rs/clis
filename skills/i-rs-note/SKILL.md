---
name: "i-rs-note"
description: "Manages notes (add/list/get/update/delete). Invoke when user needs to manage notes, todo lists, or text-based information."
---

# i-rs-note

Note management CLI tool for managing notes locally.

## Storage

- Config: `~/.config/i-rs/notes.json`

## Global Flags

- `--json` — Output in JSON format
## Commands

### add

Add a new note.

```bash
i-rs-note add <NAME> [OPTIONS]
```

Options:
- `-t, --title <TITLE>` - Note title
- `-T, --tag <TAG>` - Tags (can be repeated)
- `-c, --content <CONTENT>` - Content lines (can be repeated)

### list

List notes.

```bash
i-rs-note list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get note details.

```bash
i-rs-note get <NAME>
```

### update

Update note.

```bash
i-rs-note update <NAME>
```

Options:
- `-t, --title <TITLE>` - New title
- `-T, --tag <TAG>` - New tags
- `-c, --content <CONTENT>` - New content lines

### delete

Delete note.

```bash
i-rs-note delete <NAME>
```

### data

Manage data (export, import, clear).

```bash
i-rs-note data export
i-rs-note data import [FILE]
i-rs-note data clear
```

### example

Show usage examples.

```bash
i-rs-note example
```

### skill

Show skill information.

```bash
i-rs-note skill [summary|content|raw]
```

## Examples

```bash
# Add a note
i-rs-note add todo --title "Todo List" --tag work --content "Buy groceries" [OPTIONS]

# List all notes
i-rs-note list [OPTIONS]

# Get note
i-rs-note get todo

# Update note
i-rs-note update todo --content "New task"

# Delete note
i-rs-note delete todo
```
