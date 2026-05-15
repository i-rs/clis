---
name: "i-rs-sub"
description: "Tracks subscription services and renewal reminders. Invoke when user wants to manage recurring subscriptions."
---

# i-rs-sub

Subscription tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/sub.json`

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Add a subscription.

```bash
i-rs-sub add <NAME> <AMOUNT> <CURRENCY> <CYCLE> <START_DATE> [OPTIONS]
```

Arguments:
- `NAME` - Subscription name
- `AMOUNT` - Billing amount
- `CURRENCY` - Currency (e.g., CNY, USD)
- `CYCLE` - Billing cycle (monthly, yearly, etc.)
- `START_DATE` - Start/next billing date (YYYY-MM-DD)

Options:
- `-u, --url <URL>` - Service URL
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List subscriptions.

```bash
i-rs-sub list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get subscription details.

```bash
i-rs-sub get <NAME>
```

### update

Update a subscription.

```bash
i-rs-sub update <NAME> [OPTIONS]
```

Options:
- `-a, --amount <AMOUNT>` - New amount
- `--cycle <CYCLE>` - New billing cycle
- `--next-date <DATE>` - Next billing date
- `-u, --url <URL>` - New URL
- `-t, --tag <TAG>` - New tags
- `-r, --remark <REMARK>` - New remarks

### delete

Delete a subscription.

```bash
i-rs-sub delete <NAME>
```

### data

Manage data (export, import, clear).

```bash
i-rs-sub data export
i-rs-sub data import [FILE]
i-rs-sub data clear
```

### example

Show usage examples.

```bash
i-rs-sub example
```

### skill

Show skill information.

```bash
i-rs-sub skill [summary|content|raw]
```

## Examples

```bash
# Add a subscription
i-rs-sub add Netflix 15.99 USD monthly 2024-01-01

# With URL and tags
i-rs-sub add Spotify 9.99 USD monthly 2024-01-15 --url "https://spotify.com" --tag music

# List subscriptions
i-rs-sub list

# Update billing
i-rs-sub update Netflix --amount 19.99
```
