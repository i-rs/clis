# i-rs-gift Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Add a new gift record.

```bash
i-rs-gift add <NAME> <TYPE> <RECIPIENT> <OCCASION> <VALUE> <DATE> [OPTIONS]
```

**Arguments:**
- `NAME`: Gift name
- `TYPE`: Gift type (`sent` or `received`)
- `RECIPIENT`: Person who gave/received the gift
- `OCCASION`: Occasion (birthday, christmas, anniversary, etc.)
- `VALUE`: Monetary value
- `DATE`: Date in YYYY-MM-DD format

**Options:**
- `-t, --tag <TAG>`: Add tags (repeatable)
- `-r, --remark <REMARK>`: Add remarks (repeatable)

### list

List all gifts or filter by type/tag.

```bash
i-rs-gift list [OPTIONS]
```

**Options:**
- `-t, --type <TYPE>`: Filter by type (`sent` or `received`)
- `--tag <TAG>`: Filter by tag

### get

Get detailed information about a gift.

```bash
i-rs-gift get <NAME>
```

### delete

Delete a gift record.

```bash
i-rs-gift delete <NAME>
```

### stats

Show gift statistics including:
- Total gifts sent/received
- Total value sent/received
- Average values
- Balance (received - sent)
- Top occasions
- Top recipients

```bash
i-rs-gift stats
```

### data

Manage data (export, import, clear).

```bash
i-rs-gift data export
i-rs-gift data import [FILE]
i-rs-gift data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data

### example

Show usage examples.

```bash
i-rs-gift example
```

### skill

Show skill information.

```bash
i-rs-gift skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/gifts.json`
- Linux: `~/.config/i-rs/gifts.json`
- Windows: `~\AppData\Roaming\i-rs\gifts.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-gift list
```
