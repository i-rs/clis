# i-rs-appliance

Home appliance lifecycle management CLI tool for tracking appliances, maintenance records, and replacement reminders.

## Features

- Record appliances with brand, model, and purchase date
- Track expected lifespan in years
- Add and view maintenance history
- Automatic expiry status (OK, SOON, EXPIRED)
- Tag support for organization
- Statistics overview
- JSON output support

## Install

```bash
npm install -g @i-rs/i-rs-appliance
# or
brew install i-rs/homebrew-tap/i-rs-appliance
```

## Quick Start

```bash
# Add an appliance
i-rs-appliance add "Refrigerator" Samsung "RF28R7551" 2020-01-15 10 --tag kitchen

# List all appliances
i-rs-appliance list

# Show appliance details
i-rs-appliance get "Refrigerator"

# Add maintenance record
i-rs-appliance update "Refrigerator" --add-maintenance "Cleaned coils"

# View statistics
i-rs-appliance stats

# Delete appliance
i-rs-appliance delete "Old Microwave"
```

## Commands

### add
Add a new appliance with lifespan tracking.

```bash
i-rs-appliance add <NAME> <BRAND> <MODEL> <PURCHASE_DATE> <LIFESPAN_YEARS> [OPTIONS]
```

Options:
- `--tag, -t` - Add tags (repeatable)
- `--remark, -r` - Add remarks (repeatable)

### list
List all appliances with status overview.

```bash
i-rs-appliance list [OPTIONS]
```

Options:
- `--tag, -t` - Filter by tag

### get
Show detailed appliance information including maintenance history.

```bash
i-rs-appliance get <NAME>
```

### update
Update appliance information or add maintenance records.

```bash
i-rs-appliance update <NAME> [OPTIONS]
```

Options:
- `--brand, -b` - Update brand
- `--model, -m` - Update model
- `--lifespan, -l` - Update lifespan years
- `--tag, -t` - Update tags
- `--remark, -r` - Update remarks
- `--add-maintenance` - Add maintenance record
- `--maintenance-date` - Maintenance record date

### delete
Remove an appliance from tracking.

```bash
i-rs-appliance delete <NAME>
```

### stats
Display statistics overview.

```bash
i-rs-appliance stats
```

### example
Show usage examples.

```bash
i-rs-appliance example
```

### skill
View AI skill documentation.

```bash
i-rs-appliance skill          # Show raw skill document
i-rs-appliance skill summary  # Show summary
i-rs-appliance skill content  # Show content
```

## Data Storage

- macOS: `~/.config/i-rs/appliances.json`
- Linux: `~/.config/i-rs/appliances.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Status Indicators

| Status | Meaning |
|--------|---------|
| OK | Good condition, plenty of time before expiry |
| SOON | Within 90 days of expected lifespan |
| EXPIRED | Past expected lifespan |

## License

MIT OR Apache-2.0
