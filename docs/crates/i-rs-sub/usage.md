# i-rs-sub Usage

## Commands

### add

Add a subscription.

```bash
i-rs-sub add <NAME> --amount <AMOUNT> --cycle <CYCLE> --next-date <DATE> [OPTIONS]
```

Arguments:
- `NAME` - Subscription name

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

## Data Storage

- macOS: `~/.config/i-rs/sub.json`
- Linux: `~/.config/i-rs/sub.json`
- Windows: `~\AppData\Roaming\i-rs\sub.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-sub list
```