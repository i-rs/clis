# i-rs-recur Usage

## Commands

### add

Add a recurring expense.

```bash
i-rs-recur add <NAME> --amount <AMOUNT> --frequency <FREQ> [OPTIONS]
```

Arguments:
- `NAME` - Expense name

Options:
- `--amount <AMOUNT>` - Amount
- `--currency <CURRENCY>` - Currency (default: CNY)
- `--frequency <FREQ>` - Frequency (daily, weekly, monthly, quarterly, yearly)
- `--start-date <DATE>` - Start date (YYYY-MM-DD)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List recurring expenses.

```bash
i-rs-recur list
```

### get

Get expense details.

```bash
i-rs-recur get <NAME>
```

### delete

Delete an expense.

```bash
i-rs-recur delete <NAME>
```

### update

Update an expense.

```bash
i-rs-recur update <NAME> [OPTIONS]
```

Options:
- `--amount <AMOUNT>` - Update amount
- `--frequency <FREQ>` - Update frequency
- `-t, --tag <TAG>` - Add tags
- `-r, --remark <REMARK>` - Add remarks

## Frequency Options

- `daily` - Every day
- `weekly` - Every week
- `monthly` - Every month
- `quarterly` - Every 3 months
- `yearly` - Every year

## Data Storage

- macOS: `~/.config/i-rs/recur.json`
- Linux: `~/.config/i-rs/recur.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-recur list
```