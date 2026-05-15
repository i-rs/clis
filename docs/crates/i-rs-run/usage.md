# i-rs-run Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Add a new running record.

```bash
i-rs-run add <DATE> <DISTANCE> <DURATION> [OPTIONS]
```

**Arguments:**
- `DATE` - Date in YYYY-MM-DD format
- `DISTANCE` - Distance in km
- `DURATION` - Duration in minutes

**Options:**
- `--heart-rate <BPM>` - Average heart rate
- `-w, --weather <COND>` - Weather condition
- `-t, --tags <TAG>` - Tags (repeatable)
- `-r, --remark <TEXT>` - Remarks (repeatable)

### list

List running records.

```bash
i-rs-run list
```

### get

Get record details.

```bash
i-rs-run get <ID>
```

### delete

Delete a record.

```bash
i-rs-run delete <ID>
```

### update

Update a record.

```bash
i-rs-run update <ID> [OPTIONS]
```

### stats

Show running statistics.

```bash
i-rs-run stats
```

### plan-add

Add a training plan.

```bash
i-rs-run plan-add <NAME> <TARGET> <PACE> [OPTIONS]
```

### plan-list

List training plans.

```bash
i-rs-run plan-list
```

### plan-get

Get plan details.

```bash
i-rs-run plan-get <ID>
```

### plan-delete

Delete a plan.

```bash
i-rs-run plan-delete <ID>
```

### data

Manage data (export, import, clear).

```bash
i-rs-run data export
i-rs-run data import [FILE]
i-rs-run data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data

### example

Show usage examples.

```bash
i-rs-run example
```

### skill

Show skill information.

```bash
i-rs-run skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/runs.json`
- Linux: `~/.config/i-rs/runs.json`
- Windows: `~\AppData\Roaming\i-rs\runs.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-run list
```
