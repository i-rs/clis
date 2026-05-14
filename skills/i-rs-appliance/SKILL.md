---
name: "i-rs-appliance"
description: "Manages home appliance lifecycle (add/list/get/update/delete). Invoke when tracking appliance lifespans, recording maintenance, or setting replacement reminders."
---

# i-rs-appliance

Home appliance lifecycle management CLI tool for tracking appliances, maintenance records, and replacement reminders.

## Storage

- Config: `~/.config/i-rs/appliances.json`

## Commands

### add

Add a new appliance.

```bash
i-rs-appliance add <NAME> <BRAND> <MODEL> <PURCHASE_DATE> <LIFESPAN_YEARS> [--tag] [--remark]
```

### list

List all appliances.

```bash
i-rs-appliance list [--tag TAG]
```

### get

Show appliance details.

```bash
i-rs-appliance get <NAME>
```

### update

Update appliance or add maintenance.

```bash
i-rs-appliance update <NAME> [--brand] [--model] [--lifespan] [--tag] [--remark] [--add-maintenance] [--maintenance-date]
```

### delete

Delete appliance.

```bash
i-rs-appliance delete <NAME>
```

### stats

Show statistics.

```bash
i-rs-appliance stats
```

## Examples

```bash
# Add appliance
i-rs-appliance add "Refrigerator" Samsung "RF28R7551" 2020-01-15 10 --tag kitchen

# List appliances
i-rs-appliance list
i-rs-appliance list --tag kitchen

# Get details
i-rs-appliance get "Refrigerator"

# Add maintenance
i-rs-appliance update "Refrigerator" --add-maintenance "Cleaned coils"

# View stats
i-rs-appliance stats
```
