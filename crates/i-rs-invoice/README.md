# i-rs-invoice

Invoice management CLI tool for tracking expense invoices and reimbursement status.

## Features

- Add electronic and paper invoices
- Track reimbursement status
- Tag-based categorization
- Statistics and reporting
- JSON output support

## Install

```bash
cargo build -p i-rs-invoice
```

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

## Commands

| Command | Description |
|---------|-------------|
| `add` | Add a new invoice |
| `list` | List all invoices |
| `get` | Get invoice details |
| `update` | Update an invoice |
| `delete` | Delete an invoice |
| `stats` | View statistics |
| `example` | Show usage examples |
| `skill` | Show AI skill documentation |

## Data Storage

- macOS: `~/.config/i-rs/invoice.json`
- Linux: `~/.config/i-rs/invoice.json`
- Windows: `~\AppData\Roaming\i-rs\invoice.json`

## License

MIT OR Apache-2.0
