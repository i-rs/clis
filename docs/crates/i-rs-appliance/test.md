# i-rs-appliance Test Records

## Test Data Setup

```bash
# Add test appliances
i-rs-appliance add "Test Fridge" Samsung "TEST001" 2020-01-15 10 --tag test
i-rs-appliance add "Test Washer" LG "TEST002" 2021-06-20 8 --tag test
i-rs-appliance add "Test AC" Daikin "TEST003" 2019-03-10 5 --tag test
```

## Command Tests

### Add Command

```bash
i-rs-appliance add "Test Microwave" Panasonic "NN-TEST" 2022-01-05 8 --tag test --remark "For testing"
```

### List Command

```bash
i-rs-appliance list
i-rs-appliance list --tag test
```

### Get Command

```bash
i-rs-appliance get "Test Fridge"
```

### Update Command

```bash
i-rs-appliance update "Test Fridge" --add-maintenance "Test maintenance"
i-rs-appliance update "Test Fridge" --lifespan 12
```

### Delete Command

```bash
i-rs-appliance delete "Test Microwave"
```

### Stats Command

```bash
i-rs-appliance stats
```

### Example Command

```bash
i-rs-appliance example
```

### Skill Command

```bash
i-rs-appliance skill
i-rs-appliance skill summary
i-rs-appliance skill content
```

### JSON Output

```bash
i-rs-appliance list --json
i-rs-appliance get "Test Washer" --json
```

## Cleanup Test Data

```bash
i-rs-appliance delete "Test Fridge"
i-rs-appliance delete "Test Washer"
i-rs-appliance delete "Test AC"
```
