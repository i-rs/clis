# i-rs-towel Examples

## Basic Usage

### Recording Replacements

```bash
# Bath towels
i-rs-towel add bath
i-rs-towel add bath --tag guest-bathroom

# Face towels
i-rs-towel add face
i-rs-towel add face --tag master-bathroom

# Hand towels
i-rs-towel add hand
```

### With Tags

```bash
# By bathroom
i-rs-towel add bath --tag master-bathroom
i-rs-towel add bath --tag guest-bathroom

# By frequency
i-rs-towel add face --tag yearly
i-rs-towel add hand --tag yearly
```

## Viewing Records

```bash
# List all records
i-rs-towel list

# Filter by tag
i-rs-towel list --tag master-bathroom
```

## Managing Records

```bash
# Get record details
i-rs-towel get abc12345

# Delete a record
i-rs-towel delete abc12345
```

## Schedule Tracking

```bash
# Annual replacement
i-rs-towel add bath --tag annual --remark "New set"
i-rs-towel add face --tag annual

# Beach season
i-rs-towel add beach --tag summer --remark "New beach towel"
```