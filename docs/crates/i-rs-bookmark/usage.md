# i-rs-bookmark Usage Guide

## Commands Overview

| Command | Description |
|---------|-------------|
| `add` | Add a new bookmark |
| `list` | List all bookmarks |
| `get` | Get bookmark details |
| `update` | Update a bookmark |
| `delete` | Delete a bookmark |

## add

Add a new bookmark.

```bash
i-rs-bookmark add <NAME> <URL> [OPTIONS]
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `NAME` | Bookmark name (unique identifier) | Yes |
| `URL` | Website URL | Yes |

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-u` | `--account` | Account/username (optional) |
| `-p` | `--password` | Password (stored securely in keychain, optional) |
| `-t` | `--tag` | Tags (can be specified multiple times) |
| `-r` | `--remark` | Remarks (can be specified multiple times) |

---

## list

List all bookmarks, optionally filtered by tag.

```bash
i-rs-bookmark list [OPTIONS]
```

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-t` | `--tag` | Filter by tag |

---

## get

Get detailed information about a bookmark.

```bash
i-rs-bookmark get <NAME> [OPTIONS]
```

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-s` | `--show-password` | Show password from keychain |

---

## update

Update an existing bookmark.

```bash
i-rs-bookmark update <NAME> [OPTIONS]
```

### Options

| Short | Long | Description |
|-------|------|-------------|
| | `--url` | New URL |
| `-u` | `--account` | New account |
| `-p` | `--password` | New password (stored in keychain) |
| `-t` | `--tag` | New tags |
| `-r` | `--remark` | New remarks |

---

## delete

Delete a bookmark.

```bash
i-rs-bookmark delete <NAME>
```