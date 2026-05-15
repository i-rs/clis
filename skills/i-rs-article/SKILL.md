---
name: "i-rs-article"
description: "Article read-later management. Invoke when user wants to save articles for later reading, track reading status, or organize reading lists."
---

# i-rs-article

Article read-later CLI tool for saving and organizing articles for future reading.

## Storage

- Config: `~/.config/i-rs/articles.json`

## Global Flags

- `--json` — Output in JSON format

## Commands

### add
Add a new article.
```bash
i-rs-article add <NAME> <URL> <TITLE> [--source] [--tag] [--remark]
```

### list
List all articles or filter by tag/status.
```bash
i-rs-article list [--tag TAG] [--status STATUS]
```

### get
Get article details.
```bash
i-rs-article get <NAME>
```

### read
Mark article as read.
```bash
i-rs-article read <NAME>
```

### update
Update article information.
```bash
i-rs-article update <NAME> [--title] [--url] [--source] [--status] [--tag] [--remark] [--notes]
```

### delete
Delete an article.
```bash
i-rs-article delete <NAME>
```

### stats
Show reading statistics.
```bash
i-rs-article stats
```

### data

Manage data (export, import, clear).

```bash
i-rs-article data export
i-rs-article data import [FILE]
i-rs-article data clear
```

### example

Show usage examples.

```bash
i-rs-article example
```

### skill

Show skill information.

```bash
i-rs-article skill [summary|content|raw]
```

## Examples

```bash
# Add an article
i-rs-article add rust-blog https://rust-lang.org/blog "Rust Blog" --source rust-lang.org --tag programming [OPTIONS]

# List unread articles
i-rs-article list --status unread

# Mark as read
i-rs-article read rust-blog

# View statistics
i-rs-article stats
```
