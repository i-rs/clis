# i-rs-petbath

Pet bath tracking CLI tool for recording when you bathe your pets.

## Overview

i-rs-petbath helps you track pet bathing schedules. Maintain hygiene by keeping track of when your pets had baths.

## Quick Start

```bash
# Record pet bath
i-rs-petbath add "Cat"
i-rs-petbath add "Dog" --tag summer

# List all records
i-rs-petbath list

# Get record details
i-rs-petbath get abc12345

# Delete a record
i-rs-petbath delete abc12345
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-petbath

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-petbath
```

## Data Storage

- macOS: `~/.config/i-rs/petbath.json`
- Linux: `~/.config/i-rs/petbath.json`
- Windows: `~\AppData\Roaming\i-rs\petbath.json`

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records