---
name: "i-rs-plant"
description: "Plant care tracking CLI tool. Invoke when managing plants, tracking watering schedules, or need plant care reminders."
---

# i-rs-plant

Plant care tracking CLI tool for managing indoor and outdoor plants.

## Storage

- Config: `~/.config/i-rs/plant.json`

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
