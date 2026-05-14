# i-rs-bed Examples

## Basic Usage

### Recording Replacements

```bash
# Mattress
i-rs-bed add mattress
i-rs-bed add mattress --tag guest-room

# Pillows
i-rs-bed add pillow
i-rs-bed add pillow --tag backup

# Duvet
i-rs-bed add duvet
```

### With Tags

```bash
# By room
i-rs-bed add pillow --tag master-bedroom
i-rs-bed add pillow --tag guest-room

# By frequency
i-rs-bed add mattress-protector --tag annual
```

## Viewing Records

```bash
# List all records
i-rs-bed list

# Filter by tag
i-rs-bed list --tag master-bedroom
```

## Managing Records

```bash
# Get record details
i-rs-bed get abc12345

# Delete a record
i-rs-bed delete abc12345
```

## Schedule Tracking

```bash
# Mattress replacement
i-rs-bed add mattress --tag long-term --remark "Memory foam"

# Pillow update
i-rs-bed add pillow --tag annual --remark "Memory foam pillow"
i-rs-bed add pillow --tag quarterly --remark "Down alternative"

# Seasonal
i-rs-bed add duvet --tag seasonal --remark "Switched to light summer duvet"
```