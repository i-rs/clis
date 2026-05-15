# i-rs-sub Usage

## Global Flags

- `--json` — Output in JSON format

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