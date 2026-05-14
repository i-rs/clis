# i-rs-petbath Examples

## Basic Usage

### Recording Baths

```bash
# Simple recording
i-rs-petbath add "Cat"
i-rs-petbath add "Dog"

# With tags
i-rs-petbath add "Buddy" --tag summer
i-rs-petbath add "Whiskers" --tag spring-cleaning
```

### With Remarks

```bash
# With remarks
i-rs-petbath add "Dog" --remark "Used oatmeal shampoo"
i-rs-petbath add "Cat" --remark "Short fur trim after bath"
```

## Viewing Records

```bash
# List all records
i-rs-petbath list

# Filter by tag
i-rs-petbath list --tag summer
```

## Managing Records

```bash
# Get record details
i-rs-petbath get abc12345

# Delete a record
i-rs-petbath delete abc12345
```

## Schedule Tracking

```bash
# Regular grooming
i-rs-petbath add "Buddy" --tag monthly --remark "Full groom"
i-rs-petbath add "Whiskers" --tag bi-monthly
```