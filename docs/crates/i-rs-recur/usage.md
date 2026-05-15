# i-rs-recur Usage

## Global Flags

- `--json` — Output in JSON format

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

### data

Manage data (export, import, clear).

```bash
i-rs-recur data export
i-rs-recur data import [FILE]
i-rs-recur data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
### example

Show usage examples.

```bash
i-rs-recur example
```
### skill

Show skill information.

```bash
i-rs-recur skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/recur.json`
- Linux: `~/.config/i-rs/recur.json`
- Windows: `~\AppData\Roaming\i-rs\recur.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-recur list
```