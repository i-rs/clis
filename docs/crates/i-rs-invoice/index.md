# i-rs-invoice

Invoice management CLI tool for tracking expense invoices and reimbursement status.

## Features

- Add electronic and paper invoices
- Track reimbursement status
- Tag-based categorization
- Statistics and reporting
- JSON output support

## Quick Start

```bash
# Add an electronic invoice
i-rs-invoice add "Office Supplies" --amount 299.99 --type electronic --tags expense

# Add a paper invoice
i-rs-invoice add "Travel Expense" --amount 1500.00 --type paper --tags travel

# List all invoices
i-rs-invoice list

# View statistics
i-rs-invoice stats

# Mark as reimbursed
i-rs-invoice update <id> --reimbursed
```

## Data Storage

- Config: `~/.config/i-rs/invoice.json`
- Can be overridden with `CONFIG_DIR` environment variable
