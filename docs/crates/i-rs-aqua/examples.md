# i-rs-aqua Examples

## Basic Usage

### Recording Water Changes

```bash
# Simple recording
i-rs-aqua add

# With tank size
i-rs-aqua add --tank-size 100
i-rs-aqua add --tank-size 200
```

### With Tags

```bash
# Weekly maintenance
i-rs-aqua add --tag weekly

# Partial change
i-rs-aqua add --tag partial --tag 30-percent
```

## Viewing Records

```bash
# List all records
i-rs-aqua list

# Filter by tag
i-rs-aqua list --tag weekly
```

## Managing Records

```bash
# Get record details
i-rs-aqua get abc12345

# Delete a record
i-rs-aqua delete abc12345
```

## Maintenance Schedule

```bash
# Weekly 30% change
i-rs-aqua add --tank-size 100 --tag weekly --remark "30% water change"

# Bi-weekly full maintenance
i-rs-aqua add --tank-size 200 --tag bi-weekly --remark "Full gravel clean"
```