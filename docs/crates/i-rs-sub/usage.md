# i-rs-sub Usage

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

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data

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

## Data Storage

- macOS: `~/.config/i-rs/sub.json`
- Linux: `~/.config/i-rs/sub.json`
- Windows: `~\AppData\Roaming\i-rs\sub.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-sub list
```
