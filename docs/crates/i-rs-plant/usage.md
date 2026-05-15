# i-rs-plant Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Add a new plant.

```bash
i-rs-plant add <NAME> <SPECIES> <LOCATION> <WATERING_INTERVAL_DAYS> [OPTIONS]
```

**Arguments:**
- `NAME` - Plant name
- `SPECIES` - Plant species
- `LOCATION` - Plant location
- `WATERING_INTERVAL_DAYS` - Watering interval in days

**Options:**
- `-t, --tag <TAG>` - Tags (repeatable)
- `-r, --remark <REMARK>` - Remarks (repeatable)

### list

List all plants.

```bash
i-rs-plant list [OPTIONS]
```

**Options:**
- `-t, --tag <TAG>` - Filter by tag

### water

Water a plant.

```bash
i-rs-plant water <NAME>
```

### get

Get plant details.

```bash
i-rs-plant get <NAME>
```

### update

Update plant information.

```bash
i-rs-plant update <NAME> [OPTIONS]
```

### delete

Delete a plant.

```bash
i-rs-plant delete <NAME>
```

### stats

Show plant care statistics.

```bash
i-rs-plant stats
```

### data

Manage data (export, import, clear).

```bash
i-rs-plant data export
i-rs-plant data import [FILE]
i-rs-plant data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data

### example

Show usage examples.

```bash
i-rs-plant example
```

### skill

Show skill information.

```bash
i-rs-plant skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/plants.json`
- Linux: `~/.config/i-rs/plants.json`
- Windows: `~\AppData\Roaming\i-rs\plants.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-plant list
```
