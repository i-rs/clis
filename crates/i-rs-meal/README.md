# i-rs-meal

Daily meal tracking CLI tool for recording breakfast, lunch, and dinner.

## Features

- Track meal types (breakfast, lunch, dinner, snack)
- Record food items
- Optional calorie tracking
- Tag support for categorization

## Install

```bash
npm install -g @i-rs/i-rs-meal
# or
brew install i-rs/homebrew-tap/i-rs-meal
```

## Quick Start

```bash
# Record meals
i-rs-meal add breakfast --food "Oatmeal with berries"
i-rs-meal add lunch --food "Salad with chicken"
i-rs-meal add dinner --food "Pasta with seafood"

# With calories
i-rs-meal add lunch --food "Grilled fish" --calories 350

# List meals for a date
i-rs-meal list
```

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/meal.json`
- Linux: `~/.config/i-rs/meal.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0