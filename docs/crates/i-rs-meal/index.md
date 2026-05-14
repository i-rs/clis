# i-rs-meal

Daily meal tracking CLI tool for recording breakfast, lunch, and dinner.

## Overview

i-rs-meal helps you track your daily meals. Record what you eat, when you eat, and optionally track calories.

## Quick Start

```bash
# Record meals
i-rs-meal add breakfast --food "Oatmeal with berries"
i-rs-meal add lunch --food "Salad with chicken"

# List meals
i-rs-meal list
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-meal

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-meal
```

## Data Storage

- macOS: `~/.config/i-rs/meal.json`
- Linux: `~/.config/i-rs/meal.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Features

- **Meal Types**: breakfast, lunch, dinner, snack
- **Food Tracking**: Record food items
- **Calorie Tracking**: Optional calorie count
- **Tag Support**: Categorize entries with tags

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records