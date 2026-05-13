# i-rs-note Usage Guide

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