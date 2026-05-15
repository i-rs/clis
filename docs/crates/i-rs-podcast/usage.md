# i-rs-podcast Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Add a new podcast or course.

```bash
i-rs-podcast add <NAME> [OPTIONS]
```

**Options:**
- `-a, --author <TEXT>` - Author/host name
- `-d, --duration <SECONDS>` - Total duration in seconds
- `-t, --tag <TAG>` - Tags (repeatable)
- `--remark <TEXT>` - Remark lines (repeatable)
- `--notes <TEXT>` - Note lines (repeatable)

**Example:**
```bash
i-rs-podcast add "The Daily" --author "NYT" --duration 3600
i-rs-podcast add "Rust Course" --duration 7200 --tag rust --tag programming
```

### list

List all podcasts with optional filters.

```bash
i-rs-podcast list [OPTIONS]
```

**Options:**
- `--status <STATUS>` - Filter by status (not_started, in_progress, completed)
- `-t, --tag <TAG>` - Filter by tag

**Example:**
```bash
i-rs-podcast list
i-rs-podcast list --status in_progress
i-rs-podcast list --tag programming
```

### get

Show detailed podcast information.

```bash
i-rs-podcast get <NAME>
```

**Example:**
```bash
i-rs-podcast get "The Daily"
```

### listen

Update listening progress with current position.

```bash
i-rs-podcast listen <NAME> --position <SECONDS> [OPTIONS]
```

**Options:**
- `-p, --position <SECONDS>` - Current position in seconds
- `--notes <TEXT>` - Add note lines (repeatable)

**Example:**
```bash
i-rs-podcast listen "The Daily" --position 1800
i-rs-podcast listen "Rust Course" --position 3600 --notes "Key concept: ownership"
```

### update

Update podcast information.

```bash
i-rs-podcast update <NAME> [OPTIONS]
```

**Options:**
- `-a, --author <TEXT>` - Author/host name
- `-d, --duration <SECONDS>` - Total duration in seconds
- `-t, --tag <TAG>` - Tags (repeatable)
- `--remark <TEXT>` - Remark lines (repeatable)
- `--notes <TEXT>` - Note lines (repeatable)

**Example:**
```bash
i-rs-podcast update "The Daily" --author "NYT"
i-rs-podcast update "Rust Course" --tag tutorial
```

### delete

Delete a podcast from collection.

```bash
i-rs-podcast delete <NAME>
```

**Example:**
```bash
i-rs-podcast delete "The Daily"
```

### stats

Show podcast collection statistics.

```bash
i-rs-podcast stats
```

### data

Manage data (export, import, clear).

```bash
i-rs-podcast data export
i-rs-podcast data import [FILE]
i-rs-podcast data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data

### example

Show usage examples.

```bash
i-rs-podcast example
```

### skill

Show skill information.

```bash
i-rs-podcast skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/podcasts.json`
- Linux: `~/.config/i-rs/podcasts.json`
- Windows: `~\AppData\Roaming\i-rs\podcasts.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-podcast list
```
