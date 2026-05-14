# i-rs-appliance Examples

## Adding Appliances

### Basic Appliance

```bash
i-rs-appliance add "Refrigerator" Samsung "RF28R7551" 2020-01-15 10
```

### With Tags and Remarks

```bash
i-rs-appliance add "Washing Machine" LG "WM4000" 2021-06-20 8 \
  --tag laundry \
  --tag white-goods \
  --remark "Front loader, 9kg capacity"
```

### Kitchen Appliances Batch

```bash
i-rs-appliance add "Dishwasher" Bosch "SHP878" 2019-03-10 10 --tag kitchen
i-rs-appliance add "Microwave" Panasonic "NN-SN966" 2022-01-05 8 --tag kitchen
i-rs-appliance add "Range Hood" Broan "BCS" 2018-07-22 12 --tag kitchen
```

## Listing Appliances

### List All Appliances

```bash
i-rs-appliance list
```

### Filter by Tag

```bash
i-rs-appliance list --tag kitchen
i-rs-appliance list --tag laundry
```

## Viewing Details

### Get Appliance Details

```bash
i-rs-appliance get "Refrigerator"
```

### JSON Output

```bash
i-rs-appliance get "Refrigerator" --json
```

## Updating Appliances

### Update Lifespan

```bash
i-rs-appliance update "Refrigerator" --lifespan 12
```

### Update Tags

```bash
i-rs-appliance update "Refrigerator" --tag kitchen --tag appliances
```

### Add Maintenance Record

```bash
i-rs-appliance update "Refrigerator" --add-maintenance "Cleaned condenser coils"
```

### Add Maintenance with Date

```bash
i-rs-appliance update "Washing Machine" \
  --add-maintenance "Replaced drain pump" \
  --maintenance-date 2024-03-15
```

### Multiple Updates

```bash
i-rs-appliance update "Refrigerator" \
  --lifespan 15 \
  --remark "Extended warranty purchased" \
  --add-maintenance "Annual maintenance completed"
```

## Deleting Appliances

### Delete by Name

```bash
i-rs-appliance delete "Old Microwave"
```

## Statistics

### View Statistics

```bash
i-rs-appliance stats
```

Shows:
- Total appliances count
- Expired appliances count
- Appliances needing replacement soon (within 90 days)
- Healthy appliances count
- Percentage breakdown

## Workflow Examples

### Annual Appliance Check

```bash
# List all appliances
i-rs-appliance list

# Check each appliance for maintenance
i-rs-appliance get "Refrigerator"
i-rs-appliance get "Washing Machine"
i-rs-appliance get "Dishwasher"

# Add maintenance records
i-rs-appliance update "Refrigerator" --add-maintenance "Annual inspection"
i-rs-appliance update "Washing Machine" --add-maintenance "Cleaned gasket"

# View updated stats
i-rs-appliance stats
```

### Finding Items Needing Replacement

```bash
# List all and look for EXPIRED or SOON status
i-rs-appliance list

# Or view stats to see counts
i-rs-appliance stats
```

### Moving/Relocating Checklist

```bash
# List all appliances by location
i-rs-appliance list --tag kitchen
i-rs-appliance list --tag laundry

# Check which need replacement before moving
i-rs-appliance stats
```
