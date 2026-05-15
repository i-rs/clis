# i-rs-cycling Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Add a new cycling record.

```bash
i-rs-cycling add <DATE> <DISTANCE> <DURATION> [OPTIONS]
```

**Arguments:**
- `DATE` - Date in YYYY-MM-DD format
- `DISTANCE` - Distance in kilometers (f64)
- `DURATION` - Duration in minutes (u32)

**Options:**
- `-e, --elevation <METERS>` - Elevation gain in meters
- `-r, --route <TEXT>` - Route description
- `-t, --tag <TAG>` - Tags (repeatable)
- `-m, --remark <TEXT>` - Remarks (repeatable)

**Examples:**
```bash
i-rs-cycling add 2025-06-14 25.5 60 --elevation 300
i-rs-cycling add 2025-06-15 30.2 75 --route "Mountain Trail" --tag mountain
```

### list

List all cycling records.

```bash
i-rs-cycling list [OPTIONS]
```

**Options:**
- `-t, --tag <TAG>` - Filter by tag

**Examples:**
```bash
i-rs-cycling list
i-rs-cycling list --tag mountain
```

### get

View record details.

```bash
i-rs-cycling get <ID|DATE>
```

**Arguments:**
- `ID|DATE` - Record UUID or date

**Examples:**
```bash
i-rs-cycling get 550e8400-e29b-41d4-a716-446655440000
i-rs-cycling get 2025-06-14
```

### update

Update an existing record.

```bash
i-rs-cycling update <ID|DATE> [OPTIONS]
```

**Arguments:**
- `ID|DATE` - Record UUID or date

**Options:**
- `-d, --distance <KM>` - Update distance
- `-u, --duration <MIN>` - Update duration
- `-e, --elevation <METERS>` - Update/set elevation
- `-r, --route <TEXT>` - Update route
- `--add-tag <TAG>` - Add a tag
- `--remove-tag <TAG>` - Remove a tag
- `--add-remark <TEXT>` - Add a remark

**Examples:**
```bash
i-rs-cycling update <uuid> --distance 26.0
i-rs-cycling update 2025-06-14 --add-tag favorite
```

### delete

Delete a cycling record.

```bash
i-rs-cycling delete <ID|DATE>
```

**Arguments:**
- `ID|DATE` - Record UUID or date

**Examples:**
```bash
i-rs-cycling delete 550e8400-e29b-41d4-a716-446655440000
i-rs-cycling delete 2025-06-14
```

### stats

View cumulative statistics.

```bash
i-rs-cycling stats
```

Displays:
- Total number of records
- Total distance (km)
- Total duration (minutes and hours)
- Total elevation gain (meters)
- Average speed (km/h)
- Average distance per ride
- Average duration per ride

### Global Options

- `--json` - Output in JSON format

### data

Manage data (export, import, clear).

```bash
i-rs-cycling data export
i-rs-cycling data import [FILE]
i-rs-cycling data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
### example

Show usage examples.

```bash
i-rs-cycling example
```
### skill

Show skill information.

```bash
i-rs-cycling skill [summary|content|raw]
```
