# i-rs-toothbrush Examples

## Basic Usage

### Recording Replacements

```bash
# Electric toothbrush heads
i-rs-toothbrush add "Electric"
i-rs-toothbrush add "Electric" --tag bathroom

# Manual toothbrushes
i-rs-toothbrush add "Manual"
i-rs-toothbrush add "Manual" --tag travel
```

### With Tags

```bash
# By location
i-rs-toothbrush add "Electric" --tag master-bathroom
i-rs-toothbrush add "Electric" --tag guest-bathroom

# By type
i-rs-toothbrush add "Manual" --tag backup
i-rs-toothbrush add "Kids" --tag children
```

## Viewing Records

```bash
# List all records
i-rs-toothbrush list

# Filter by tag
i-rs-toothbrush list --tag bathroom
```

## Managing Records

```bash
# Get record details
i-rs-toothbrush get abc12345

# Delete a record
i-rs-toothbrush delete abc12345
```

## Schedule Tracking

```bash
# Quarterly replacement
i-rs-toothbrush add "Electric" --tag quarterly --remark "Changed head"
i-rs-toothbrush add "Manual" --tag quarterly

# Travel backup
i-rs-toothbrush add "Manual" --tag travel --remark "New travel brush"
```