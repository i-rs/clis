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
- `HEIGHT` - Height in cm

**Options:**
- `-w, --weight <KG>` - Weight in kg
- `-t, --tag <TAG>` - Tags (repeatable)
- `-r, --remark <TEXT>` - Remarks (repeatable)

### list

List all height records.

```bash
i-rs-height list [OPTIONS]
```

**Options:**
- `-d, --days <N>` - Show records from last N days
- `-c, --chart` - Show chart
- `-s, --stats` - Show statistics

### get

Get specific record.

```bash
i-rs-height get <DATE>
```

### delete

Delete a record.

```bash
i-rs-height delete <DATE>
```

### set

Set a height target.

```bash
i-rs-height set <HEIGHT>
```

### target

Show current height target.

```bash
i-rs-height target
```

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

### example

Show usage examples.

```bash
i-rs-height example
```

### skill

Show skill information.

```bash
i-rs-height skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/height.json`
- Linux: `~/.config/i-rs/height.json`
- Windows: `~\AppData\Roaming\i-rs\height.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-height list
```
