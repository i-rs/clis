# i-rs-toothbrush

Toothbrush replacement tracking CLI tool for recording when you replace toothbrushes.

## Overview

i-rs-toothbrush helps you track toothbrush replacement schedules. Maintain oral hygiene by replacing toothbrushes regularly.

## Quick Start

```bash
# Record toothbrush replacement
i-rs-toothbrush add "Electric"
i-rs-toothbrush add "Manual" --tag travel

# List all records
i-rs-toothbrush list

# Get record details
i-rs-toothbrush get abc12345

# Delete a record
i-rs-toothbrush delete abc12345
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-toothbrush

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-toothbrush
```

## Data Storage

- macOS: `~/.config/i-rs/toothbrushes.json`
- Linux: `~/.config/i-rs/toothbrushes.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Brush Types

- `Electric` - Electric toothbrush head
- `Manual` - Regular manual toothbrush
- `Kids` - Children's toothbrush
- `Interdental` - Interdental brush

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records