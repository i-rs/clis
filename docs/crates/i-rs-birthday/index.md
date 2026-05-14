# i-rs-birthday

Birthday reminder CLI tool for managing birthdays and never missing an important date.

## Overview

i-rs-birthday helps you track birthdays of friends, family, and colleagues. It automatically calculates ages, shows days until next birthday, and provides statistics and insights.

## Quick Start

```bash
# Add a birthday
i-rs-birthday add John 06-15 --year 1990 --relationship friend --tag personal

# Add family member
i-rs-birthday add Mom 08-20 --year 1965 --relationship family --tag important

# List all birthdays
i-rs-birthday list

# List by tag
i-rs-birthday list --tag family

# Get details
i-rs-birthday get John

# View upcoming birthdays
i-rs-birthday upcoming

# View statistics
i-rs-birthday stats

# JSON output
i-rs-birthday list --json
```

## Features

- Add, update, delete, and list birthdays
- Track relationships (family, friend, colleague, etc.)
- Automatic age calculation
- Days until birthday countdown
- Upcoming birthdays view
- Statistics and insights
- Tag support for organization
- JSON output support

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-birthday

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-birthday
```

## Data Storage

- macOS: `~/.config/i-rs/birthdays.json`
- Linux: `~/.config/i-rs/birthdays.json`
- Windows: `~\AppData\Roaming\i-rs\birthday.json`

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records
