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
i-rs-sub add <NAME> --amount <AMOUNT> --cycle <CYCLE> --next-date <DATE> [OPTIONS]
```

Options:
- `--amount <AMOUNT>` - Billing amount
- `--currency <CURRENCY>` - Currency (default: CNY)
- `--cycle <CYCLE>` - Billing cycle (monthly, yearly, etc.)
- `--next-date <DATE>` - Next billing date (YYYY-MM-DD)
- `--url <URL>` - Service URL
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List subscriptions.

```bash
i-rs-sub list
```

### get

Get subscription details.

```bash
i-rs-sub get <NAME>
```

### delete

Delete a subscription.

```bash
i-rs-sub delete <NAME>
```

### update

Update a subscription.

```bash
i-rs-sub update <NAME> [OPTIONS]
```

Options:
- `--amount <AMOUNT>` - Update amount
- `--next-date <DATE>` - Update next billing date
- `--url <URL>` - Update URL
- `-t, --tag <TAG>` - Add tags
- `-r, --remark <REMARK>` - Add remarks

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
# Add subscription
i-rs-sub add "Netflix" --amount 15.99 --cycle monthly --next-date 2024-02-15
i-rs-sub add "Spotify" --amount 9.99 --cycle monthly --next-date 2024-02-20

# List subscriptions
i-rs-sub list

# Get details
i-rs-sub get Netflix
```