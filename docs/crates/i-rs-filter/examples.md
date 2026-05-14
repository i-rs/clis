# i-rs-filter Examples

## Basic Usage

### Air Purifier Filters

```bash
# HEPA filters
i-rs-filter add "Air Purifier" HEPA
i-rs-filter add "Air Purifier" HEPA --tag living-room

# Carbon filters
i-rs-filter add "Air Purifier" Carbon
```

### Vacuum Filters

```bash
# Dust filters
i-rs-filter add "Vacuum" Dust
i-rs-filter add "Vacuum" Foam
```

### With Tags

```bash
# By room
i-rs-filter add "Air Purifier" HEPA --tag bedroom
i-rs-filter add "Air Purifier" HEPA --tag living-room

# By frequency
i-rs-filter add "Vacuum" Dust --tag monthly
```

## Viewing Records

```bash
# List all records
i-rs-filter list

# Filter by tag
i-rs-filter list --tag monthly
i-rs-filter list --tag bedroom
```

## Managing Records

```bash
# Get record details
i-rs-filter get abc12345

# Delete a record
i-rs-filter delete abc12345
```

## Schedule Tracking

```bash
# Monthly air purifier
i-rs-filter add "Air Purifier" HEPA --tag monthly --remark "Washed and dried"

# Quarterly vacuum
i-rs-filter add "Vacuum" Dust --tag quarterly
```