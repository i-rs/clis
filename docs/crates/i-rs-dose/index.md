# i-rs-dose

Medicine dosage tracking CLI tool for recording when you take medications.

## Overview

i-rs-dose helps you track your medicine intake. Record what medications you take, when you take them, and maintain a history for reference.

## Quick Start

```bash
# Record medicine intake
i-rs-dose add "Vitamin D" --dosage 1000 --unit IU

# List all records
i-rs-dose list

# Get record details
i-rs-dose get abc12345
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-dose

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-dose
```

## Data Storage

- macOS: `~/.config/i-rs/dose.json`
- Linux: `~/.config/i-rs/dose.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Features

- **Dosage Tracking**: Record medicine name and dosage amount
- **Unit Support**: Various units (tablet, ml, mg, IU, etc.)
- **Time History**: Automatic timestamp for each entry
- **Tag Support**: Categorize entries with tags

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records