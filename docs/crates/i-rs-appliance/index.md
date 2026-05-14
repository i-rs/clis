# i-rs-appliance

Home appliance lifecycle management CLI tool for tracking appliances, maintenance records, and replacement reminders.

## Overview

i-rs-appliance helps you keep track of home appliances, their expected lifespan, and maintenance history. Never miss a replacement deadline again.

## Features

- Record appliances with brand, model, and purchase date
- Track expected lifespan in years
- Add and view maintenance history
- Automatic expiry status calculation
- Tag support for organization
- Statistics overview
- JSON output for scripting

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-appliance

# Homebrew
brew install i-rs/homebrew-tap/i-rs-appliance
```

## Quick Start

```bash
# Add your first appliance
i-rs-appliance add "Refrigerator" Samsung "RF28R7551" 2020-01-15 10 --tag kitchen

# List all appliances
i-rs-appliance list

# Check appliance details
i-rs-appliance get "Refrigerator"

# Add maintenance record
i-rs-appliance update "Refrigerator" --add-maintenance "Cleaned coils"

# View statistics
i-rs-appliance stats
```

## Data Storage

Data is stored in JSON format:
- macOS: `~/.config/i-rs/appliances.json`
- Linux: `~/.config/i-rs/appliances.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

Override with `CONFIG_DIR` environment variable.
