# i-rs-sub

Subscription tracking CLI tool for managing recurring subscriptions and renewal reminders.

## Overview

i-rs-sub helps you track subscription services and know when they need to be renewed.

## Quick Start

```bash
# Add subscription
i-rs-sub add "Netflix" --amount 15.99 --cycle monthly --next-date 2024-02-15

# List all
i-rs-sub list
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-sub

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-sub
```

## Data Storage

- macOS: `~/.config/i-rs/sub.json`
- Linux: `~/.config/i-rs/sub.json`
- Windows: `~\AppData\Roaming\i-rs\sub.json`

## Features

- **Subscription Tracking**: Track service subscriptions
- **Billing Cycles**: Monthly, yearly, etc.
- **Renewal Countdown**: Shows days until next billing
- **Status Indicators**: OK, DUE_SOON, EXPIRED

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records