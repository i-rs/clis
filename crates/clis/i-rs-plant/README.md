# i-rs-plant

Plant care tracking CLI tool for managing your indoor and outdoor plants.

## Features

- Add plants with name, species, location, and watering interval
- Track last watered date and days until next watering
- Tag support for organizing plants
- Care notes and remarks
- Statistics dashboard
- JSON output for scripting

## Install

```bash
npm install -g @i-rs/i-rs-plant
# or
brew install i-rs/homebrew-tap/i-rs-plant
```

## Quick Start

```bash
# Add a new plant
i-rs-plant add --name "Monstera" --species "Monstera deliciosa" --location "Living room" --interval 7

# List all plants
i-rs-plant list

# Water a plant
i-rs-plant water Monstera

# View statistics
i-rs-plant stats
```

## Data Storage

- macOS: `~/.config/i-rs/plant.json`
- Linux: `~/.config/i-rs/plant.json`
- Windows: `~\AppData\Roaming\i-rs\plant.json`

## License

MIT OR Apache-2.0
