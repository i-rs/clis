# i-rs-spark

Inspiration and ideas tracking CLI tool for capturing fleeting thoughts and creative sparks.

## Overview

i-rs-spark helps you capture inspiration and ideas before they slip away. Record thoughts, note sources, and build your idea repository.

## Quick Start

```bash
# Capture inspiration
i-rs-spark add "Use machine learning for text classification"
i-rs-spark add "New app idea" --source "Dream"

# List all sparks
i-rs-spark list
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-spark

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-spark
```

## Data Storage

- macOS: `~/.config/i-rs/spark.json`
- Linux: `~/.config/i-rs/spark.json`
- Windows: `~\AppData\Roaming\i-rs\spark.json`

## Features

- **Quick Capture**: Record ideas instantly
- **Source Tracking**: Note where inspiration came from
- **Tag Support**: Categorize ideas
- **Time History**: Automatic timestamps

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records