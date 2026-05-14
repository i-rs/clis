# i-rs-pig Examples

## Basic Usage

### Recording Cravings

```bash
# Quick recordings
i-rs-pig add "Chocolate bar"
i-rs-pig add "Bag of chips"
i-rs-pig add "Ice cream"

# With descriptions
i-rs-pig add "Pizza" --description "Extra cheese and pepperoni"
i-rs-pig add "Burger and fries" --description "Fast food lunch at McDonald's"
```

### With Tags and Remarks

```bash
# Tagged entries
i-rs-pig add "Donuts" --tag office --tag stress-eating
i-rs-pig add "Cookies" --tag baking --tag homemade

# With remarks
i-rs-pig add "Ice cream" --remark "Had a rough day"
i-rs-pig add "Chips" --remark "Movie night snack"
```

## Viewing Records

```bash
# List all
i-rs-pig list

# Filter by tag
i-rs-pig list --tag stress-eating

# Get details
i-rs-pig get abc12345
```

## Awareness Tracking

```bash
# Emotional eating
i-rs-pig add "Cake" --tag emotional --remark "Birthday celebration at work"
i-rs-pig add "Cookies" --tag emotional --tag stress --remark "Deadline stress"

# Social eating
i-rs-pig add "Pizza" --tag social --tag weekend --remark "Game night with friends"
i-rs-pig add "Popcorn" --tag social --remark "Movie date"

# Late night snacking
i-rs-pig add "Ramen" --tag late-night --tag stress --remark "Couldn't sleep, stress eating"
```