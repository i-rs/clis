# i-rs-snippet Usage

## Global Flags

- `--json` — Output in JSON format

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

### data

Manage data (export, import, clear).

```bash
i-rs-snippet data export
i-rs-snippet data import [FILE]
i-rs-snippet data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
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
