# i-rs-ac Examples

## Basic Usage

### Recording Cleanings

```bash
# Simple recording
i-rs-ac add "Living Room"
i-rs-ac add "Bedroom"

# With tags
i-rs-ac add "Living Room" --tag pre-summer
i-rs-ac add "Bedroom" --tag post-summer
```

### With Remarks

```bash
# With remarks
i-rs-ac add "Living Room" --remark "Full deep clean"
i-rs-ac add "Office" --remark "Filter replaced"
```

## Viewing Records

```bash
# List all records
i-rs-ac list

# Filter by tag
i-rs-ac list --tag pre-summer
```

## Managing Records

```bash
# Get record details
i-rs-ac get abc12345

# Delete a record
i-rs-ac delete abc12345
```

## Schedule Tracking

```bash
# Pre-summer maintenance
i-rs-ac add "Living Room" --tag pre-summer --remark "Full service"
i-rs-ac add "Bedroom" --tag pre-summer

# Mid-season check
i-rs-ac add "Living Room" --tag mid-season
```