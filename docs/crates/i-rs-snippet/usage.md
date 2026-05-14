# i-rs-snippet Usage

## Commands

### add

Add a new code snippet.

```bash
i-rs-snippet add <NAME> --language <LANG> --code <CODE> [options]
```

Options:
- `-l, --language <LANG>`: Programming language (required)
- `-c, --code <CODE>`: Code lines (repeatable, required)
- `-d, --description <DESC>`: Description lines (repeatable)
- `-g, --tag <TAG>`: Tags (repeatable)
- `-r, --remark <REMARK>`: Remarks (repeatable)

### list

List all snippets or filter by tag.

```bash
i-rs-snippet list [--tag <TAG>]
```

### search

Search snippets by query.

```bash
i-rs-snippet search <QUERY>
```

Searches in: name, language, code content, description, tags.

### get

Get snippet details.

```bash
i-rs-snippet get <NAME>
```

### copy

Copy snippet code to clipboard.

```bash
i-rs-snippet copy <NAME>
```

### update

Update a snippet.

```bash
i-rs-snippet update <NAME> [options]
```

Options:
- `-l, --language <LANG>`: Programming language
- `-c, --code <CODE>`: Code lines (repeatable)
- `-d, --description <DESC>`: Description lines (repeatable)
- `-g, --tag <TAG>`: Tags (repeatable)
- `-r, --remark <REMARK>`: Remarks (repeatable)

### delete

Delete a snippet.

```bash
i-rs-snippet delete <NAME>
```

### Global Options

- `--json`: Output in JSON format
