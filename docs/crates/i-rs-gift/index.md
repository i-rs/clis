# i-rs-gift

Gift management CLI tool for tracking gifts given and received.

## Overview

i-rs-gift helps you track gift exchanges with friends, family, and colleagues. Record gifts you've given or received with details like occasion, recipient, and value.

## Features

- **Gift Types**: Track both sent and received gifts
- **Occasions**: Label gifts by occasion (birthday, christmas, anniversary, etc.)
- **Statistics**: View gift exchange summaries and spending
- **Organization**: Use tags to organize gifts
- **JSON Output**: All commands support `--json` flag for programmatic access

## Quick Start

```bash
# Add a gift you gave
i-rs-gift add "Birthday Watch" sent "Mom" birthday 500 2024-12-25

# Add a gift you received
i-rs-gift add "AirPods" received "Boss" christmas 1200 2024-12-25

# View statistics
i-rs-gift stats

# List all gifts
i-rs-gift list
```

## Data Storage

Data is stored in `~/.config/i-rs/gifts.json`
