# i-rs-meal Examples

## Basic Usage

### Recording Meals

```bash
# Breakfast
i-rs-meal add breakfast --food "Oatmeal with berries"
i-rs-meal add breakfast --food "Eggs and toast"

# Lunch
i-rs-meal add lunch --food "Salad with chicken"
i-rs-meal add lunch --food "Sandwich and soup"

# Dinner
i-rs-meal add dinner --food "Pasta with seafood"
i-rs-meal add dinner --food "Grilled fish with vegetables"

# Snacks
i-rs-meal add snack --food "Apple"
i-rs-meal add snack --food "Yogurt"
```

### With Calories

```bash
# Track calories
i-rs-meal add lunch --food "Grilled chicken breast" --calories 350
i-rs-meal add lunch --food "Brown rice" --calories 220
i-rs-meal add dinner --food "Salmon fillet" --calories 400
```

### With Tags

```bash
# Healthy choices
i-rs-meal add lunch --food "Quinoa bowl" --calories 450 --tag healthy --tag vegetarian

# Treat days
i-rs-meal add dinner --food "Pizza" --tag cheat-day --tag weekend
```

## Viewing Records

```bash
# List today's meals
i-rs-meal list

# List specific date
i-rs-meal list --date 2024-01-15

# Get meal details
i-rs-meal get abc12345
```

## Daily Food Log

```bash
# Morning routine
i-rs-meal add breakfast --food "Greek yogurt with granola" --calories 300 --tag breakfast --tag healthy
i-rs-meal add snack --food "Handful of almonds" --calories 150 --tag snack

# Afternoon
i-rs-meal add lunch --food "Turkey sandwich" --calories 500 --tag lunch --tag work
i-rs-meal add snack --food "Protein bar" --calories 200 --tag snack --tag post-workout

# Evening
i-rs-meal add dinner --food "Stir-fried vegetables with tofu" --calories 400 --tag dinner --tag healthy
```