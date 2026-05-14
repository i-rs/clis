# Examples

## Basic Usage

### Add a simple plant

```bash
i-rs-plant add --name "Snake Plant" --species "Sansevieria" --location "Office" --interval 14
```

### Add a plant with tags and remarks

```bash
i-rs-plant add \
  --name "Fiddle Leaf Fig" \
  --species "Ficus lyrata" \
  --location "Living Room" \
  --interval 10 \
  --tag indoor \
  --tag large \
  --remark "Needs bright indirect light" \
  --remark "Wipe leaves monthly"
```

### List all plants

```bash
i-rs-plant list
```

Output:
```
┌─────────────────┬─────────────────────────────┬──────────────┬──────────┬───────────────┬─────────────┐
│ Name            │ Species                     │ Location     │ Interval │ Last Watered  │ Days Until  │
├─────────────────┼─────────────────────────────┼──────────────┼──────────┼───────────────┼─────────────┤
│ Monstera        │ Monstera deliciosa          │ Living Room  │ 7 days   │ 2024-01-01    │ 3 days      │
│ Snake Plant     │ Sansevieria                 │ Office       │ 14 days  │ 2023-12-20    │ 5 days      │
└─────────────────┴─────────────────────────────┴──────────────┴──────────┴───────────────┴─────────────┘

Total: 2 plants
```

### List plants by tag

```bash
i-rs-plant list --tag indoor
```

### Get plant details

```bash
i-rs-plant get Monstera
```

### Water a plant

```bash
i-rs-plant water Monstera
```

### View statistics

```bash
i-rs-plant stats
```

## Advanced Usage

### Update plant information

```bash
# Update location
i-rs-plant update Monstera --location "Bedroom"

# Update watering interval
i-rs-plant update Snake Plant --interval 21

# Add new tags
i-rs-plant update Monstera --tag tropical --tag pet-friendly
```

### Delete a plant

```bash
i-rs-plant delete "Dead Orchid"
```

## JSON Output

All commands support JSON output for scripting:

```bash
# List in JSON
i-rs-plant list --json

# Get in JSON
i-rs-plant get Monstera --json

# Stats in JSON
i-rs-plant stats --json
```

## Automation

### Water all plants that need it

```bash
# Using JSON output (requires jq)
for name in $(i-rs-plant list --json | jq -r '.data[] | select(.needs_water == true) | .name'); do
  i-rs-plant water "$name"
done
```

### Backup plant data

```bash
cp ~/.config/i-rs/plant.json ~/backup/plants-$(date +%Y%m%d).json
```
