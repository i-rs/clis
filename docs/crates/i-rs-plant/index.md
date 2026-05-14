# i-rs-plant

Plant care tracking CLI tool for managing your indoor and outdoor plants.

## Overview

i-rs-plant helps you keep track of your plants, their watering schedules, and care requirements. Never forget to water your plants again!

## Features

- Add plants with name, species, location, and watering interval
- Track last watered date and days until next watering
- Tag support for organizing plants by location or type
- Care notes and remarks for special instructions
- Statistics dashboard showing plant health
- JSON output for scripting and automation

## Quick Start

```bash
# Add a new plant
i-rs-plant add --name "Monstera" --species "Monstera deliciosa" --location "Living room" --interval 7

# List all plants
i-rs-plant list

# Water a plant
i-rs-plant water Monstera

# View plant details
i-rs-plant get Monstera

# View statistics
i-rs-plant stats
```

## Data Storage

Data is stored in JSON format at:

- macOS: `~/.config/i-rs/plant.json`
- Linux: `~/.config/i-rs/plant.json`
- Windows: `~\AppData\Roaming\i-rs\plant.json`
