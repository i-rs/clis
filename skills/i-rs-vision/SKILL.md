---
name: "i-rs-vision"
description: "Tracks vision prescription (add/list/get/delete). Invoke when user needs to record eye measurements, view vision history, or display statistics."
---

# i-rs-vision

Vision tracking CLI tool for recording and tracking eye prescription measurements over time.

## Storage

- Config: `~/.config/i-rs/visions.json`

## Global Flags

- `--json` — Output in JSON format
## Vision Parameters

- **Sphere**: Refractive error (negative = myopia, positive = hyperopia)
- **Cylinder**: Astigmatism correction (typically 0 to -2.00)
- **Axis**: Orientation of astigmatism (0-180 degrees)

## Commands

### add

Add a vision record.

```bash
i-rs-vision add <DATE> \
  --left-sphere <D> \
  --right-sphere <D> \
  --left-cylinder <D> \
  --right-cylinder <D> \
  --left-axis <DEG> \
  --right-axis <DEG> \
  --tag <TAG> \
  --remark <TEXT>
```

### list

List vision records.

```bash
i-rs-vision list [--days N]
```

### get

Get a specific record.

```bash
i-rs-vision get <DATE>
```

### delete

Delete a vision record.

```bash
i-rs-vision delete <DATE>
```

### stats

Show vision statistics.

```bash
i-rs-vision stats
```

## Options

| Option | Short | Description |
|--------|-------|-------------|
| --left-sphere | -l | Left eye sphere (diopters) |
| --right-sphere | -r | Right eye sphere (diopters) |
| --left-cylinder | -L | Left eye cylinder (diopters) |
| --right-cylinder | -R | Right eye cylinder (diopters) |
| --left-axis | -a | Left eye axis (degrees) |
| --right-axis | -b | Right eye axis (degrees) |
| --tag | -t | Tags (repeatable) |
| --remark | -m | Remarks (repeatable) |
| --json | | JSON output format |

### data

Manage data (export, import, clear).

```bash
i-rs-vision data export
i-rs-vision data import [FILE]
i-rs-vision data clear
```

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

## Examples

```bash
# Add basic record
i-rs-vision add 2025-06-14 -l -3.50 -r -4.00

# Add full record
i-rs-vision add 2025-06-14 -l -3.50 -r -4.00 -L -0.50 -R -0.75 -a 180 -b 5

# Add with tags
i-rs-vision add 2025-06-14 -l -3.50 -r -4.00 -t myopia -t annual

# List all records
i-rs-vision list

# List recent records
i-rs-vision list --days 30

# Get specific record
i-rs-vision get 2025-06-14

# View statistics
i-rs-vision stats

# JSON output
i-rs-vision list --json
i-rs-vision stats --json
```

## Notes

- At least one of --left-sphere or --right-sphere is required
- Negative sphere values indicate myopia (nearsightedness)
- Positive sphere values indicate hyperopia (farsightedness)
