---
name: "i-rs-snippet"
description: "Manages code snippets (add/list/get/search/copy/update/delete). Invoke when user needs to save, retrieve, or search code snippets."
---

# i-rs-snippet

Code snippet management CLI tool for storing, organizing, and quickly retrieving code snippets.

## Storage

- Config: `~/.config/i-rs/snippets.json`

## Global Flags

- `--json` — Output in JSON format
## Commands

### add

Add a new code snippet.

```bash
i-rs-snippet add <NAME> --language <LANG> --code <CODE> [--tag] [--description] [--remark]
```

### list

List all snippets or filter by tag.

```bash
i-rs-snippet list [--tag TAG]
```

### search

Search snippets by query.

```bash
i-rs-snippet search <QUERY>
```

### get

Get snippet details.

```bash
i-rs-snippet get <NAME>
```

### copy

Copy snippet to clipboard.

```bash
i-rs-snippet copy <NAME>
```

### update

Update a snippet.

```bash
i-rs-snippet update <NAME> [--language] [--code] [--tag] [--description] [--remark]
```

### delete

Delete a snippet.

```bash
i-rs-snippet delete <NAME>
```

### data

Manage data (export, import, clear).

```bash
i-rs-snippet data export
i-rs-snippet data import [FILE]
i-rs-snippet data clear
```

### example

Show usage examples.

```bash
i-rs-snippet example
```

### skill

Show skill information.

```bash
i-rs-snippet skill [summary|content|raw]
```

## Examples

```bash
# Add snippet
i-rs-snippet add hello --language rust --code 'fn main() { println!("Hello!"); }' --tag rust

# List all
i-rs-snippet list

# Search
i-rs-snippet search hello

# Copy to clipboard
i-rs-snippet copy hello
```
