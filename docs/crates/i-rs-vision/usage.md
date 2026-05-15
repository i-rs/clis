# i-rs-vision Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Add a new vision record.

```bash
i-rs-vision add <DATE> [OPTIONS]
```

**Options:**

- `-l, --left-sphere <D>` - Left eye sphere (diopters)
- `-r, --right-sphere <D>` - Right eye sphere (diopters)
- `-L, --left-cylinder <D>` - Left eye cylinder (diopters)
- `-R, --right-cylinder <D>` - Right eye cylinder (diopters)
- `-a, --left-axis <DEG>` - Left eye axis (degrees)
- `-b, --right-axis <DEG>` - Right eye axis (degrees)
- `-t, --tag <TAG>` - Tags (repeatable)
- `-r, --remark <TEXT>` - Remarks (repeatable)

**Examples:**

```bash
# Basic record
i-rs-vision add 2025-06-14 -l -3.50 -r -4.00

# Full record with cylinder and axis
i-rs-vision add 2025-06-14 -l -3.50 -r -4.00 -L -0.50 -R -0.75 -a 180 -b 5

# With tags and remarks
i-rs-vision add 2025-06-14 -l -3.50 -r -4.00 -t myopia -r "Annual checkup"
```

### list

List all vision records.

```bash
i-rs-vision list [OPTIONS]
```

**Options:**

- `-d, --days <N>` - Show records from last N days

**Examples:**

```bash
# List all records
i-rs-vision list

# List last 30 days
i-rs-vision list --days 30

# List last year
i-rs-vision list -d 365
```

### get

Get a specific vision record.

```bash
i-rs-vision get <DATE>
```

**Examples:**

```bash
i-rs-vision get 2025-06-14
```

### delete

Delete a vision record.

```bash
i-rs-vision delete <DATE>
```

**Examples:**

```bash
i-rs-vision delete 2025-06-14
```

### stats

Show vision statistics.

```bash
i-rs-vision stats
```

Displays:

- Total number of records
- Earliest and latest record dates
- Latest vision measurements
- Vision changes over time

### Global Options

- `--json` - Output in JSON format

## Understanding Vision Values

### Sphere (球镜)

Refractive error measurement:

- **Negative values**: Myopia (nearsightedness)
- **Positive values**: Hyperopia (farsightedness)
- **Typical range**: -10.00 to +6.00 diopters

### Cylinder (柱镜)

Astigmatism correction:

- Typically ranges from 0 to -2.00 diopters
- 0 means no astigmatism correction needed

### Axis (轴位)

Orientation of astigmatism:

- Range: 0-180 degrees
- Only relevant when cylinder is present

### data

Manage data (export, import, clear).

```bash
i-rs-vision data export
i-rs-vision data import [FILE]
i-rs-vision data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
### example

Show usage examples.

```bash
i-rs-vision example
```
### skill

Show skill information.

```bash
i-rs-vision skill [summary|content|raw]
```
