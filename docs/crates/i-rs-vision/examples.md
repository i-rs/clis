# i-rs-vision Examples

## Basic Usage

### Add Basic Vision Record

```bash
i-rs-vision add 2025-06-14 -l -3.50 -r -4.00
```

Output:
```
✓ Vision record added for 2025-06-14
```

### Add Full Vision Record

```bash
i-rs-vision add 2025-06-14 \
  -l -3.50 \
  -r -4.00 \
  -L -0.50 \
  -R -0.75 \
  -a 180 \
  -b 5
```

### Add with Tags and Remarks

```bash
i-rs-vision add 2025-06-14 \
  -l -3.50 \
  -r -4.00 \
  -t myopia \
  -t progressive \
  -r "Annual eye exam" \
  -r "瞳孔扩张检查"
```

## List Records

### List All Records

```bash
i-rs-vision list
```

Output:
```
┌────────────┬───────────┬───────────┬─────────┬─────────┬─────────┬─────────┬─────────┬─────────────────────┐
│ DATE       │ L.SPHERE │ R.SPHERE │ L.CYL   │ R.CYL   │ L.AXIS  │ R.AXIS  │ TAGS    │ REMARK              │
├────────────┼───────────┼───────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────────────────┤
│ 2025-06-14 │ -3.50    │ -4.00     │ -0.50   │ -0.75   │ 180     │ 5       │ myopia  │ Annual checkup     │
│ 2025-06-01 │ -3.25    │ -3.75     │ -0.50   │ -0.75   │ 180     │ 5       │ -       │ -                   │
└────────────┴───────────┴───────────┴─────────┴─────────┴─────────┴─────────┴─────────┴─────────────────────┘

Total: 2 records
```

### List Recent Records

```bash
i-rs-vision list --days 30
```

### List Last Year

```bash
i-rs-vision list -d 365
```

## Get Specific Record

```bash
i-rs-vision get 2025-06-14
```

## View Statistics

```bash
i-rs-vision stats
```

Output:
```
Vision Statistics:
────────────────────────────────────────

  Total Records:    5
  Earliest Record: 2024-06-14
  Latest Record:   2025-06-14

Latest Vision:
  Left Sphere:    -3.50
  Right Sphere:   -4.00
  Left Cylinder:  -0.50
  Right Cylinder: -0.75
  Left Axis:      180
  Right Axis:     5

Left Eye Change:
                  +0.50 (+0.50 ↑)

Right Eye Change:
                  +0.50 (+0.50 ↑)
```

## JSON Output

### List in JSON

```bash
i-rs-vision list --json
```

Output:
```json
{
  "success": true,
  "data": [
    {
      "date": "2025-06-14",
      "left_sphere": -3.50,
      "right_sphere": -4.00,
      "left_cylinder": -0.50,
      "right_cylinder": -0.75,
      "left_axis": 180,
      "right_axis": 5,
      "tags": ["myopia"],
      "remark": ["Annual checkup"]
    }
  ],
  "meta": {
    "count": 1,
    "filter": null
  }
}
```

### Stats in JSON

```bash
i-rs-vision stats --json
```

## Delete Records

```bash
i-rs-vision delete 2025-06-14
```

## Practical Scenarios

### Annual Eye Exam

```bash
# Record after annual eye exam
i-rs-vision add 2025-06-14 \
  -l -3.50 \
  -r -4.00 \
  -L -0.50 \
  -R -0.75 \
  -a 180 \
  -b 5 \
  -t annual \
  -t eye-exam \
  -r "Doctor: Smith" \
  -r "New prescription"
```

### Track Myopia Progression

```bash
# Monthly check-in
i-rs-vision add 2025-01-15 -l -3.00 -r -3.50 -t monthly
i-rs-vision add 2025-02-15 -l -3.10 -r -3.60 -t monthly
i-rs-vision add 2025-03-15 -l -3.15 -r -3.65 -t monthly

# View progression
i-rs-vision stats
```

### Presbyopia (Age-related)

```bash
# Near vision declining
i-rs-vision add 2025-06-14 \
  -l +0.50 \
  -r +0.75 \
  -L -0.25 \
  -R -0.25 \
  -a 90 \
  -b 90 \
  -t presbyopia \
  -t reading-glasses
```
