# i-rs-article Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Add a new article.

```bash
i-rs-article add <NAME> <URL> <TITLE> [OPTIONS]
```

Options:
- `--source, -s`: Article source (e.g., blog name, website)
- `--tag, -g`: Tags (repeatable)
- `--remark, -r`: Remarks (repeatable)

Example:
```bash
i-rs-article add rust-blog https://rust-lang.org/blog "Rust Blog" --source rust-lang.org --tag programming
```

### list

List all articles or filter by tag/status.

```bash
i-rs-article list [OPTIONS]
```

Options:
- `--tag, -g`: Filter by tag
- `--status`: Filter by status (unread/reading/read)

Examples:
```bash
i-rs-article list
i-rs-article list --tag programming
i-rs-article list --status unread
i-rs-article list --tag rust --status unread
```

### get

Get article details.

```bash
i-rs-article get <NAME>
```

Example:
```bash
i-rs-article get rust-blog
```

### read

Mark article as read.

```bash
i-rs-article read <NAME>
```

Example:
```bash
i-rs-article read rust-blog
```

### update

Update article information.

```bash
i-rs-article update <NAME> [OPTIONS]
```

Options:
- `--title`: Update title
- `--url`: Update URL
- `--source, -s`: Update source
- `--status`: Update status (unread/reading/read)
- `--tag, -g`: Update tags
- `--remark, -r`: Update remarks
- `--notes, -n`: Update notes

Examples:
```bash
i-rs-article update rust-blog --remark "Important article"
i-rs-article update rust-blog --status reading
i-rs-article update rust-blog --notes "Key point 1" "Key point 2"
```

### delete

Delete an article.

```bash
i-rs-article delete <NAME>
```

Example:
```bash
i-rs-article delete rust-blog
```

### stats

Show reading statistics.

```bash
i-rs-article stats
```

## Global Options

- `--json`: Output in JSON format

### data

Manage data (export, import, clear).

```bash
i-rs-article data export
i-rs-article data import [FILE]
i-rs-article data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
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
