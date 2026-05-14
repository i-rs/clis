# i-rs-sheet Examples

## Basic Usage

### Recording Changes

```bash
# Simple recording
i-rs-sheet add bedsheet
i-rs-sheet add pillowcase

# With tags
i-rs-sheet add bedsheet --tag bedroom
i-rs-sheet add pillowcase --tag guest-room
```

### Full Sheet Change

```bash
# Change all at once
i-rs-sheet add bedsheet --tag weekly-change
i-rs-sheet add pillowcase --tag weekly-change
i-rs-sheet add duvet-cover --tag weekly-change
```

## Viewing Records

```bash
# List all records
i-rs-sheet list

# Filter by tag
i-rs-sheet list --tag bedroom
i-rs-sheet list --tag weekly-change
```

## Managing Records

```bash
# Get record details
i-rs-sheet get abc12345

# Delete a record
i-rs-sheet delete abc12345
```

## Schedule Tracking

```bash
# Weekly bedroom
i-rs-sheet add bedsheet --tag bedroom --remark "Weekly change"
i-rs-sheet add pillowcase --tag bedroom --remark "Weekly change"

# Monthly
i-rs-sheet add mattress-protector --tag monthly
i-rs-sheet add duvet-cover --tag monthly
```