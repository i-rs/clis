# i-rs-allergy

Allergy tracking CLI tool for recording allergic reactions and symptoms.

## Overview

i-rs-allergy helps you track allergic reactions. Record allergens, severity, symptoms, and maintain a history for medical reference.

## Quick Start

```bash
# Record allergy reaction
i-rs-allergy add "Peanuts" mild --symptom hives --symptom itching
i-rs-allergy add "Pollen" severe --symptom sneezing

# List all records
i-rs-allergy list

# Get record details
i-rs-allergy get abc12345

# Delete a record
i-rs-allergy delete abc12345
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-allergy

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-allergy
```

## Data Storage

- macOS: `~/.config/i-rs/allergies.json`
- Linux: `~/.config/i-rs/allergies.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Severity Levels

- `mild` - Minor reaction
- `moderate` - Noticeable reaction
- `severe` - Serious reaction

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records