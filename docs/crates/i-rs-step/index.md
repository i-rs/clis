# i-rs-step

Step counting CLI tool for tracking daily walking and running steps.

## Overview

i-rs-step helps you track your daily step count and walking/running distance.

## Quick Start

```bash
# Record steps
i-rs-step add 10000
i-rs-step add 8000 --distance 6.4

# List all records
i-rs-step list
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-step

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-step
```

## Data Storage

- macOS: `~/.config/i-rs/step.json`
- Linux: `~/.config/i-rs/step.json`
- Windows: `~\AppData\Roaming\i-rs\step.json`

## Features

- **Step Tracking**: Record daily step count
- **Distance Tracking**: Optional distance in kilometers
- **Statistics**: Track totals over time
- **Tag Support**: Categorize entries

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records