# i-rs-car Examples

## Basic Operations

### Add a new car

```bash
i-rs-car add "My Car" --license-plate "ABC123" --brand "Toyota" --model "Camry" --mileage 50000
```

### Add a car with tags

```bash
i-rs-car add "Family SUV" --license-plate "XYZ789" --brand "Honda" --model "CR-V" --mileage 30000 --tags family,travel
```

### List all cars

```bash
i-rs-car list
```

### List specific car

```bash
i-rs-car list --car "My Car"
```

### Get car details

```bash
i-rs-car get "My Car"
```

### Update car mileage

```bash
i-rs-car update "My Car" --mileage 55000
```

### Update car tags

```bash
i-rs-car update "My Car" --add-tags business
```

### Delete a car

```bash
i-rs-car delete "My Car" --force
```

## Fuel Records

### Add fuel record

```bash
i-rs-car fuel "My Car" --mileage 51000 --fuel-amount 45 --price 8.5
```

### Add fuel record with details

```bash
i-rs-car fuel "My Car" --mileage 52000 --fuel-amount 50 --price 8.3 --fuel-type 95 --station "Shell Station" --note "Highway trip"
```

### View fuel records

```bash
i-rs-car get "My Car" --fuel
```

## Maintenance Records

### Add maintenance record

```bash
i-rs-car maintain "My Car" --mileage 52000 --maintenance-type oil_change --cost 300
```

### Add maintenance with shop info

```bash
i-rs-car maintain "My Car" --mileage 55000 --maintenance-type tire_rotation --cost 150 --shop "Toyota 4S" --description "Seasonal tire rotation"
```

### Add brake maintenance

```bash
i-rs-car maintain "My Car" --mileage 60000 --maintenance-type brake --cost 1200 --shop "Auto Service Center"
```

### View maintenance records

```bash
i-rs-car get "My Car" --maintain
```

## Statistics

### View overall statistics

```bash
i-rs-car stats
```

### View car-specific statistics

```bash
i-rs-car stats --car "My Car"
```

## JSON Output

### List in JSON format

```bash
i-rs-car list --json
```

### Get details in JSON format

```bash
i-rs-car get "My Car" --json
```

### Fuel records in JSON format

```bash
i-rs-car get "My Car" --fuel --json
```

### Statistics in JSON format

```bash
i-rs-car stats --json
```
