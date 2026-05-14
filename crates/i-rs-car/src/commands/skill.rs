use anyhow::Result;
use clap::Parser;

const SKILL_CONTENT: &str = r#"---
name: "i-rs-car"
description: "Vehicle management CLI tool. Track car information, fuel records, maintenance history, and mileage statistics."
---

# i-rs-car

Vehicle management CLI tool for tracking car information, fuel records, maintenance history, and statistics.

## Storage

- Config: `~/.config/i-rs/cars.json`

## Commands

### add
Create a new car entry.
```bash
i-rs-car add <name> [options]
```
Options:
- `--license-plate`: License plate number
- `--brand`: Car brand (e.g., Toyota, Honda, BMW)
- `--model`: Car model (e.g., Camry, Civic, X5)
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

## JSON Output

All commands support `--json` flag for JSON output:
```bash
i-rs-car list --json
i-rs-car get <name> --json
i-rs-car stats --json
```
"#;

const SKILL_SUMMARY: &str = r#"i-rs-car: Vehicle management CLI for tracking car information, fuel records, maintenance history, and mileage statistics.

Commands: add, list, get, update, delete, fuel, maintain, stats, example, skill
"#;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long, help = "Show summary only")]
    pub summary: bool,
}

pub fn run(args: &Args) -> Result<()> {
    if args.summary {
        println!("{}", SKILL_SUMMARY);
    } else {
        println!("{}", SKILL_CONTENT);
    }
    Ok(())
}
