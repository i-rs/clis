# i-rs-tick

Duration tracking CLI tool for recording time spent on activities, tasks, and projects.

## Overview

i-rs-tick helps you track how long activities take. Perfect for time tracking, project estimation, and productivity analysis.

## Quick Start

```bash
# Record task duration
i-rs-tick add "Meeting" --duration 3600
i-rs-tick add "Coding" --duration 7200

# List all records
i-rs-tick list
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-tick

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-tick
```

## Data Storage

- macOS: `~/.config/i-rs/tick.json`
- Linux: `~/.config/i-rs/tick.json`
- Windows: `~\AppData\Roaming\i-rs\tick.json`

## Features

- **Duration Tracking**: Record time spent in seconds
- **Task Names**: Name your activities
- **Duration Formatting**: Human-readable format
- **Tag Support**: Categorize entries

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records