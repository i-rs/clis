# i-rs-height Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Add a new height record.

```bash
i-rs-height add <DATE> <HEIGHT> [OPTIONS]
```

**Arguments:**
- `DATE` - Date in YYYY-MM-DD format
- `HEIGHT` - Height in centimeters

**Options:**
- `-w, --weight <KG>` - Weight in kilograms (optional)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <TEXT>` - Remarks (can be repeated)

**Examples:**
```bash
i-rs-height add 2025-06-14 175.5
i-rs-height add 2025-06-14 175.5 --weight 68.5
i-rs-height add 2025-06-14 175.5 --tag "morning" --remark "After breakfast"
```

### list

List all height records.

```bash
i-rs-height list [OPTIONS]
```

**Options:**
- `-d, --days <N>` - Show records from last N days
- `-c, --chart` - Display ASCII chart
- `-s, --stats` - Show statistics

**Examples:**
```bash
i-rs-height list
i-rs-height list --days 30
i-rs-height list --chart
i-rs-height list --days 7 --chart --stats
```

### get

Get details of a specific record.

```bash
i-rs-height get <DATE>
```

**Examples:**
```bash
i-rs-height get 2025-06-14
```

### delete

Delete a height record.

```bash
i-rs-height delete <DATE>
```

**Examples:**
```bash
i-rs-height delete 2025-06-15
```

### set

Set target height.

```bash
i-rs-height set <HEIGHT>
```

**Examples:**
```bash
i-rs-height set 180.0
```

### target

Show current target height.

```bash
i-rs-height target
```

### example

Show usage examples.

```bash
i-rs-height example
```

### skill

View AI skill documentation.

```bash
i-rs-height skill [SUB_COMMAND]
```

**Sub commands:**
- `summary` - Show skill summary
- `content` - Show skill content
- `raw` - Show raw skill document

## Global Options

- `--json` - Output in JSON format
- `-h, --help` - Show help information
- `-V, --version` - Show version information

### data

Manage data (export, import, clear).

```bash
i-rs-height data export
i-rs-height data import [FILE]
i-rs-height data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
