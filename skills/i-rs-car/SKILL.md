---
name: "i-rs-car"
description: "Vehicle management CLI tool. Track car information, fuel records, maintenance history, and mileage statistics."
---

# i-rs-car

Vehicle management CLI tool for tracking car information, fuel records, maintenance history, and statistics.

## Storage

- Config: `~/.config/i-rs/car.json`

## Global Flags

- `--json` — Output in JSON format

## Commands

### add
Create a new car entry.
```bash
i-rs-car add <NAME> [OPTIONS]
```
Options:
- `-l, --license-plate <PLATE>` - License plate number
- `-b, --brand <BRAND>` - Car brand
- `-m, --model <MODEL>` - Car model
- `-i, --mileage <KM>` - Current mileage in km
- `-t, --tags <TAGS>` - Comma-separated tags
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list
List all cars.
```bash
i-rs-car list [options]
```
Options:
- `--car`: Filter by car name

### get
Show car details or records.
```bash
i-rs-car get <name> [options]
```
Options:
- `--fuel`: Show fuel records
- `--maintain`: Show maintenance records

### update
Update car information.
```bash
i-rs-car update <name> [options]
```
Options:
- `--rename`: New car name
- `--license-plate`: New license plate
- `--brand`: New brand
- `--model`: New model
- `--mileage`: New mileage
- `--add-tags`: Add tags (comma-separated)
- `--remove-tags`: Remove tags (comma-separated)
- `--add-remark`: Add remarks

### delete
Delete a car entry.
```bash
i-rs-car delete <name> [options]
```
Options:
- `--force`: Skip confirmation

### fuel
Add a fuel record.
```bash
i-rs-car fuel <car> [options]
```
Options:
- `--date`: Fuel date YYYY-MM-DD (default: today)
- `--mileage`: Current mileage (km)
- `--fuel-amount`: Fuel amount in liters
- `--price`: Price per liter
- `--fuel-type`: Fuel type (e.g., 92, 95, 98, diesel)
- `--station`: Gas station name (optional)
- `--note`: Note (optional)

### maintain
Add a maintenance record.
```bash
i-rs-car maintain <car> [options]
```
Options:
- `--date`: Maintenance date YYYY-MM-DD (default: today)
- `--mileage`: Current mileage (km)
- `--maintenance-type`: Type (e.g., oil_change, tire, brake, inspection)
- `--cost`: Maintenance cost
- `--description`: Description (optional)
- `--shop`: Shop name (optional)
- `--note`: Note (optional)

### stats
Show statistics.
```bash
i-rs-car stats [options]
```
Options:
- `--car`: Show statistics for specific car

### example
Show usage examples.
```bash
i-rs-car example
```

### skill
Show skill documentation.
```bash
i-rs-car skill
i-rs-car skill summary
```

### data

Manage data (export, import, clear).

```bash
i-rs-car data export
i-rs-car data import [FILE]
i-rs-car data clear
```

## Examples

```bash
# Add a car
i-rs-car add "My Car" --license-plate "ABC123" --brand "Toyota" --model "Camry" --mileage 50000 [OPTIONS]

# Add fuel record
i-rs-car fuel "My Car" --mileage 51000 --fuel-amount 45 --price 8.5

# Add maintenance record
i-rs-car maintain "My Car" --mileage 52000 --maintenance-type oil_change --cost 300

# View statistics
i-rs-car stats
```
