# i-rs-car Test Records

## Setup Test Cars

```bash
# Add first car
i-rs-car add "Test Car 1" --license-plate "TEST001" --brand "Toyota" --model "Corolla" --mileage 10000

# Add second car
i-rs-car add "Test Car 2" --license-plate "TEST002" --brand "Honda" --model "Civic" --mileage 20000
```

## Add Fuel Records

```bash
# Fuel for Test Car 1
i-rs-car fuel "Test Car 1" --date 2026-01-15 --mileage 10500 --fuel-amount 40 --price 8.50 --fuel-type 95 --station "Shell Station"

i-rs-car fuel "Test Car 1" --date 2026-02-15 --mileage 11000 --fuel-amount 42 --price 8.60 --fuel-type 95

i-rs-car fuel "Test Car 1" --date 2026-03-15 --mileage 11500 --fuel-amount 45 --price 8.70 --fuel-type 95

# Fuel for Test Car 2
i-rs-car fuel "Test Car 2" --date 2026-01-20 --mileage 20500 --fuel-amount 38 --price 8.40 --fuel-type 92

i-rs-car fuel "Test Car 2" --date 2026-02-20 --mileage 21000 --fuel-amount 40 --price 8.50 --fuel-type 92
```

## Add Maintenance Records

```bash
# Maintenance for Test Car 1
i-rs-car maintain "Test Car 1" --date 2026-01-01 --mileage 10000 --maintenance-type oil_change --cost 300 --shop "Toyota 4S"

i-rs-car maintain "Test Car 1" --date 2026-03-01 --mileage 11500 --maintenance-type inspection --cost 200 --shop "Toyota 4S"

# Maintenance for Test Car 2
i-rs-car maintain "Test Car 2" --date 2026-02-01 --mileage 20500 --maintenance-type tire_rotation --cost 150 --shop "Honda Service"

i-rs-car maintain "Test Car 2" --date 2026-03-15 --mileage 21000 --maintenance-type brake --cost 800 --shop "Auto Service"
```

## Verify Data

```bash
# List all cars
i-rs-car list

# Get details for Test Car 1
i-rs-car get "Test Car 1"

# Get fuel records for Test Car 1
i-rs-car get "Test Car 1" --fuel

# Get maintenance records for Test Car 1
i-rs-car get "Test Car 1" --maintain

# View statistics
i-rs-car stats

# View statistics for specific car
i-rs-car stats --car "Test Car 1"
```

## Cleanup

```bash
# Delete test cars
i-rs-car delete "Test Car 1" --force
i-rs-car delete "Test Car 2" --force
```
