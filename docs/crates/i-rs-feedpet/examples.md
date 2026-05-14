# i-rs-feedpet Examples

## Basic Usage

### Feeding Cats

```bash
# Dry food
i-rs-feedpet add "Whiskers" dry-food "50g"
i-rs-feedpet add "Whiskers" dry-food "50g" --tag morning
i-rs-feedpet add "Whiskers" dry-food "50g" --tag evening

# Wet food
i-rs-feedpet add "Whiskers" wet-food "100g" --tag special
```

### Feeding Dogs

```bash
# Regular feeding
i-rs-feedpet add "Buddy" dry-food "200g"
i-rs-feedpet add "Buddy" wet-food "200g" --tag lunch

# With treats
i-rs-feedpet add "Buddy" treat "20g" --tag training
```

## Viewing Records

```bash
# List all records
i-rs-feedpet list

# Filter by tag
i-rs-feedpet list --tag morning
i-rs-feedpet list --tag training
```

## Managing Records

```bash
# Get record details
i-rs-feedpet get abc12345

# Delete a record
i-rs-feedpet delete abc12345
```

## Multi-pet Household

```bash
# Cat
i-rs-feedpet add "Whiskers" dry-food "50g" --tag morning
i-rs-feedpet add "Whiskers" wet-food "100g" --tag evening

# Dog
i-rs-feedpet add "Buddy" dry-food "200g" --tag morning
i-rs-feedpet add "Buddy" dry-food "200g" --tag evening
```