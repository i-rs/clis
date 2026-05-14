# i-rs-car Usage

## Commands

### add

Create a new car entry.

```bash
i-rs-car add <name> [options]
```

Options:
- `--license-plate`, `-l`: License plate number
- `--brand`, `-b`: Car brand (e.g., Toyota, Honda, BMW)
- `--model`, `-m`: Car model (e.g., Camry, Civic, X5)
- `--mileage`: Current mileage in km
- `--tags`: Comma-separated tags (optional)
- `--remark`: Remarks (optional, multiple)

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
- `--license-plate`, `-l`: New license plate
- `--brand`, `-b`: New brand
- `--model`, `-m`: New model
- `--mileage`, `-m`: New mileage
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
- `--mileage`, `-m`: Current mileage (km)
- `--fuel-amount`, `-f`: Fuel amount in liters
- `--price`, `-p`: Price per liter
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
- `--mileage`, `-m`: Current mileage (km)
- `--maintenance-type`, `-t`: Type (e.g., oil_change, tire, brake, inspection)
- `--cost`, `-c`: Maintenance cost
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

## Global Options

- `--json`: Output in JSON format
