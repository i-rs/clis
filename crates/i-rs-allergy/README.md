# i-rs-allergy

Allergy tracking CLI tool for recording allergic reactions and symptoms.

## Features

- Track allergen and severity
- Record symptoms
- Time-based history
- Tag support for categorization

## Install

```bash
npm install -g @i-rs/i-rs-allergy
# or
brew install i-rs/homebrew-tap/i-rs-allergy
```

## Quick Start

```bash
# Record allergy reaction
i-rs-allergy add "Peanuts" mild --symptom hives --symptom itching
i-rs-allergy add "Pollen" severe --symptom sneezing

# List all records
i-rs-allergy list

# Get record details
i-rs-allergy get abc12345
```

## Severity Levels

- `mild` - Minor reaction
- `moderate` - Noticeable reaction
- `severe` - Serious reaction requiring attention

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/allergies.json`
- Linux: `~/.config/i-rs/allergies.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0