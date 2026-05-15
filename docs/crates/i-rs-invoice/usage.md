# i-rs-invoice Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Add a new invoice.

```bash
i-rs-invoice add <name> --amount <amount> [options]
```

Options:
- `--date, -d`: Invoice date (YYYY-MM-DD), defaults to today
- `--type, -t`: Invoice type: `electronic` or `paper` (default: electronic)
- `--reimbursed, -r`: Mark as reimbursed
- `--tags`: Add tags (can be specified multiple times)
- `--remark`: Add remarks (can be specified multiple times)
- `--json`: Output in JSON format

### list

List all invoices.

```bash
i-rs-invoice list [options]
```

Options:
- `--tag`: Filter by tag
- `--reimbursed`: Show only reimbursed invoices
- `--unreimbursed`: Show only unreimbursed invoices
- `--json`: Output in JSON format

### get

Get invoice details.

```bash
i-rs-invoice get <id> [options]
```

Options:
- `--json`: Output in JSON format

### update

Update an invoice.

```bash
i-rs-invoice update <id> [options]
```

Options:
- `--name`: Update name
- `--amount`: Update amount
- `--date`: Update date (YYYY-MM-DD)
- `--type`: Update invoice type
- `--reimbursed`: Update reimbursement status
- `--add-tags`: Add tags
- `--remove-tags`: Remove tags
- `--add-remark`: Add remarks
- `--json`: Output in JSON format

### delete

Delete an invoice.

```bash
i-rs-invoice delete <id> [options]
```

Options:
- `--json`: Output in JSON format

### stats

View invoice statistics.

```bash
i-rs-invoice stats [options]
```

Options:
- `--tag`: Filter by tag
- `--reimbursed`: Show only reimbursed statistics
- `--unreimbursed`: Show only unreimbursed statistics

### example

Show usage examples.

```bash
i-rs-invoice example
```

### skill

Show AI skill documentation.

```bash
i-rs-invoice skill [options]
```

Options:
- `--summary`: Show summary only
- `--content`: Show content only

### data

Manage data (export, import, clear).

```bash
i-rs-invoice data export
i-rs-invoice data import [FILE]
i-rs-invoice data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
