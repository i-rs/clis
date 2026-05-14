# i-rs-vision Test Records

Use these test records to verify your i-rs-vision installation.

## Setup

```bash
# Clear existing data (optional)
rm ~/.config/i-rs/visions.json
```

## Test Commands

### Add Test Records

```bash
# Record 1: Initial measurement
i-rs-vision add 2025-01-01 -l -2.50 -r -2.75 -L -0.50 -R -0.50 -a 180 -b 180 -t initial -r "First measurement"

# Record 2: 3 months later
i-rs-vision add 2025-04-01 -l -2.75 -r -3.00 -L -0.50 -R -0.50 -a 175 -b 175 -t myopia -r "Slight progression"

# Record 3: 6 months later
i-rs-vision add 2025-07-01 -l -3.00 -r -3.25 -L -0.50 -R -0.75 -a 175 -b 180 -t myopia -r "Continued progression"

# Record 4: 1 year later
i-rs-vision add 2026-01-01 -l -3.25 -r -3.50 -L -0.50 -R -0.75 -a 170 -b 180 -t myopia -r "Annual checkup"
```

### List All Records

```bash
i-rs-vision list
```

Expected output:
```
┌────────────┬───────────┬───────────┬─────────┬─────────┬─────────┬─────────┬─────────┬─────────────────────┐
│ DATE       │ L.SPHERE │ R.SPHERE │ L.CYL   │ R.CYL   │ L.AXIS  │ R.AXIS  │ TAGS    │ REMARK              │
├────────────┼───────────┼───────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────────────────┤
│ 2025-01-01 │ -2.50    │ -2.75     │ -0.50   │ -0.50   │ 180     │ 180     │ initial │ First measurement   │
│ 2025-04-01 │ -2.75    │ -3.00     │ -0.50   │ -0.50   │ 175     │ 175     │ myopia  │ Slight progression   │
│ 2025-07-01 │ -3.00    │ -3.25     │ -0.50   │ -0.75   │ 175     │ 180     │ myopia  │ Continued progressio│
│ 2026-01-01 │ -3.25    │ -3.50     │ -0.50   │ -0.75   │ 170     │ 180     │ myopia  │ Annual checkup      │
└────────────┴───────────┴───────────┴─────────┴─────────┴─────────┴─────────┴─────────┴─────────────────────┘

Total: 4 records
```

### Get Specific Record

```bash
i-rs-vision get 2025-07-01
```

### View Statistics

```bash
i-rs-vision stats
```

Expected output:
```
Vision Statistics:
────────────────────────────────────────

  Total Records:    4
  Earliest Record: 2025-01-01
  Latest Record:   2026-01-01

Latest Vision:
  Left Sphere:    -3.25
  Right Sphere:   -3.50
  Left Cylinder:  -0.50
  Right Cylinder: -0.75
  Left Axis:      170
  Right Axis:     180

Left Eye Change:
                  +0.75 (+0.75 ↑)

Right Eye Change:
                  +0.75 (+0.75 ↑)
```

### JSON Output

```bash
i-rs-vision list --json
i-rs-vision get 2025-07-01 --json
i-rs-vision stats --json
```

### Delete Test Record

```bash
i-rs-vision delete 2025-04-01
```

### Filter by Days

```bash
i-rs-vision list --days 180
```

## Expected Results

- ✓ All 4 records added successfully
- ✓ List shows all records sorted by date
- ✓ Stats show progression over time
- ✓ JSON output is valid and structured
- ✓ Delete removes record from list

## Clean Up

```bash
# Remove all test data
rm ~/.config/i-rs/visions.json
```
