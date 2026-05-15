# i-rs-note Usage Guide

## Global Flags

- `--json` — Output in JSON format

## Commands Overview

| Command | Description |
|---------|-------------|
| `add` | Add a new note |
| `list` | List all notes |
| `get` | Get note details |
| `update` | Update a note |
| `delete` | Delete a note |

## add

Add a new note.

```bash
i-rs-note add <NAME> [OPTIONS]
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `NAME` | Note name (unique identifier) | Yes |

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-t` | `--title` | Note title |
| `-T` | `--tag` | Tags (can be specified multiple times) |
| `-c` | `--content` | Content lines (can be specified multiple times) |

### Examples

```bash
# Simple note
i-rs-note add shopping --content "Milk" --content "Eggs"

# Note with title and tags
i-rs-note add todo --title "Things to do" --tag work --tag personal --content "Call doctor" --content "Fix leaky faucet"
```

---

## list

List all notes, optionally filtered by tag.

```bash
i-rs-note list [OPTIONS]
```

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-t` | `--tag` | Filter by tag |

---

## get

Get detailed information about a note.

```bash
i-rs-note get <NAME>
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `NAME` | Note name | Yes |

---

## update

Update an existing note.

```bash
i-rs-note update <NAME> [OPTIONS]
```

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-t` | `--title` | New title |
| `-T` | `--tag` | New tags (replaces all) |
| `-c` | `--content` | New content lines (replaces all) |

---

## delete

Delete a note.

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

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
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

## Data Storage

- macOS: `~/.config/i-rs/note.json`
- Linux: `~/.config/i-rs/note.json`
- Windows: `~\AppData\Roaming\i-rs\note.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-note list
```
