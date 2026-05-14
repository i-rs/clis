# i-rs-sub

Subscription tracking CLI tool for managing recurring subscriptions and renewal reminders.

## Features

- Track subscription services
- Billing cycle management
- Renewal countdown
- Expiration status indicators

## Install

```bash
npm install -g @i-rs/i-rs-sub
# or
brew install i-rs/homebrew-tap/i-rs-sub
```

## Quick Start

```bash
# Add subscription
i-rs-sub add "Netflix" --amount 15.99 --cycle monthly --next-date 2024-02-15
i-rs-sub add "Spotify" --amount 9.99 --cycle monthly --next-date 2024-02-20

# List all subscriptions
i-rs-sub list

# Get subscription details
i-rs-sub get Netflix
```

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/sub.json`
- Linux: `~/.config/i-rs/sub.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0