# i-rs-project Usage

## Global Flags

- `--json` — Output in JSON format

Detailed command reference for i-rs-project.

### milestone add

Add a milestone to a project.

```bash
i-rs-project milestone add <PROJECT> <MILESTONE> [OPTIONS]
```

Options:
- `-d, --description <TEXT>` - Milestone description
- `-D, --due-date <DATE>` - Due date (YYYY-MM-DD format)

Examples:
```bash
# Basic milestone
i-rs-project milestone add my-project "v1.0"

# With due date
i-rs-project milestone add my-project "Beta Release" --due-date 2026-06-01

# With description
i-rs-project milestone add my-project "Launch" --description "Product launch" --due-date 2026-07-01
```

### milestone complete

Mark a milestone as completed.

```bash
i-rs-project milestone complete <PROJECT> <MILESTONE>
```

Example:
```bash
i-rs-project milestone complete my-project "v1.0"
```

## task

Manage project tasks.

### task add

Add a task to a project.

```bash
i-rs-project task add <PROJECT> <TASK> [OPTIONS]
```

Options:
- `-d, --description <TEXT>` - Task description
- `-t, --tag <TAG>` - Tags (can be repeated)

Examples:
```bash
# Basic task
i-rs-project task add my-project "Write docs"

# With description and tags
i-rs-project task add my-project "Review PR" --description "Review pull request" --tag code-review
```

### task complete

Mark a task as completed.

```bash
i-rs-project task complete <PROJECT> <TASK>
```

Example:
```bash
i-rs-project task complete my-project "Write docs"
```

## stats

Show project statistics.

```bash
i-rs-project stats
```

Example:
```bash
i-rs-project stats
```

## example

Show usage examples.

```bash
i-rs-project example
```

## skill

View AI skill documentation.

```bash
i-rs-project skill [SUB_COMMAND]
```

Subcommands:
- `summary` - Show skill summary
- `content` - Show full skill content
- `raw` - Show raw skill document

### data

Manage data (export, import, clear).

```bash
i-rs-project data export
i-rs-project data import [FILE]
i-rs-project data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
### example

Show usage examples.

```bash
i-rs-project example
```
### skill

Show skill information.

```bash
i-rs-project skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/project.json`
- Linux: `~/.config/i-rs/project.json`
- Windows: `~\AppData\Roaming\i-rs\project.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-project list
```
