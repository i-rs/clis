---
name: "i-rs-plant"
description: "Plant care tracking CLI tool. Invoke when managing plants, tracking watering schedules, or need plant care reminders."
---

# i-rs-plant

Plant care tracking CLI tool for managing indoor and outdoor plants.

## Storage

- Config: `~/.config/i-rs/plant.json`

## Global Flags

- `--json` — Output in JSON format
## Commands

### add
Add a new plant:
```bash
i-rs-plant add --name "Monstera" --species "Monstera deliciosa" --location "Living room" --interval 7
```

### list
List all plants:
```bash
i-rs-plant list
i-rs-plant list --tag indoor
```

### get
Get plant details:
```bash
i-rs-plant get Monstera
```

### water
Record watering:
```bash
i-rs-plant water Monstera
```

### update
Update plant info:
```bash
i-rs-plant update Monstera --location "Bedroom"
```

### delete
Delete a plant:
```bash
i-rs-plant delete Monstera
```

### stats
View plant statistics:
```bash
i-rs-plant stats
```

### data

Manage data (export, import, clear).

```bash
i-rs-plant data export
i-rs-plant data import [FILE]
i-rs-plant data clear
```

### example

Show usage examples.

```bash
i-rs-plant example
```

### skill

Show skill information.

```bash
i-rs-plant skill [summary|content|raw]
```

## Examples

```bash
# Add a plant with tags
i-rs-plant add --name "Snake Plant" --species "Sansevieria" --location "Office" --interval 14 --tag succulent

# List all plants
i-rs-plant list

# Water all plants
for plant in $(i-rs-plant list --json | jq -r '.data[].name'); do
  i-rs-plant water "$plant"
done
```
