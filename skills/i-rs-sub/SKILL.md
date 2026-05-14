---
name: "i-rs-sub"
description: "Tracks subscription services and renewal reminders. Invoke when user wants to manage recurring subscriptions."
---

# i-rs-sub

Subscription tracking CLI tool.

## Storage

- Config: `~/.config/i-rs/sub.json`

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