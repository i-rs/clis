# i-rs-car

Vehicle management CLI tool for tracking car information, fuel records, maintenance history, and statistics.

## Features

- Car information management (name, license plate, brand, model, mileage)
- Fuel record tracking with cost calculation and fuel efficiency
- Maintenance record tracking
- Mileage statistics
- Tag support
- JSON output support

## Quick Start

```bash
# Add a car
i-rs-car add "My Car" --license-plate "ABC123" --brand "Toyota" --model "Camry" --mileage 50000

# List all cars
i-rs-car list

# Add fuel record
i-rs-car fuel "My Car" --mileage 51000 --fuel-amount 45 --price 8.5

# Add maintenance record
i-rs-car maintain "My Car" --mileage 52000 --maintenance-type oil_change --cost 300

# View car details
i-rs-car get "My Car"

# View statistics
i-rs-car stats
```

## Data Storage

- macOS: `~/.config/i-rs/cars.json`
- Linux: `~/.config/i-rs/cars.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`
