# i-rs-ledger Usage

## Commands

### add

Add a ledger entry.

```bash
i-rs-ledger add [OPTIONS]
```

Options:
- `--type <TYPE>` - Entry type (income, expense, transfer)
- `--amount <AMOUNT>` - Amount
- `--currency <CURRENCY>` - Currency (default: CNY)
- `--category <CATEGORY>` - Category
- `--date <DATE>` - Date (YYYY-MM-DD)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List ledger entries.

```bash
i-rs-ledger list
```

Options:
- `--type <TYPE>` - Filter by type
- `--category <CATEGORY>` - Filter by category

### get

Get entry details.

```bash
i-rs-ledger get <ID>
```

### delete

Delete an entry.

```bash
i-rs-ledger delete <ID>
```

## Data Storage

- macOS: `~/.config/i-rs/ledger.json`
- Linux: `~/.config/i-rs/ledger.json`
- Windows: `~\AppData\Roaming\i-rs\ledger.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-ledger list
```