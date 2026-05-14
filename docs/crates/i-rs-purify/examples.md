# i-rs-purify Examples

## Basic Usage

### Recording Replacements

```bash
# Sediment filter (every 3 months)
i-rs-purify add "Sediment Filter"
i-rs-purify add "Sediment Filter" --tag kitchen

# Carbon filter (every 6 months)
i-rs-purify add "Carbon Filter"

# RO membrane (every 2 years)
i-rs-purify add "RO Membrane"
```

### With Tags

```bash
# By location
i-rs-purify add "Sediment Filter" --tag kitchen
i-rs-purify add "Sediment Filter" --tag bathroom

# By frequency
i-rs-purify add "Carbon Filter" --tag 6-month
i-rs-purify add "RO Membrane" --tag annual
```

## Viewing Records

```bash
# List all records
i-rs-purify list

# Filter by tag
i-rs-purify list --tag kitchen
i-rs-purify list --tag annual
```

## Managing Records

```bash
# Get record details
i-rs-purify get abc12345

# Delete a record
i-rs-purify delete abc12345
```

## Schedule Tracking

```bash
# 6-month routine
i-rs-purify add "Sediment Filter" --tag routine --remark "Quarterly change"
i-rs-purify add "Carbon Filter" --tag routine

# Annual
i-rs-purify add "RO Membrane" --tag annual --remark "Lasts 2-3 years"
```