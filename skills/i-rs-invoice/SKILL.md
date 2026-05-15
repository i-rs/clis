---
name: "i-rs-invoice"
description: "Invoice management CLI tool. Invoke when managing invoices, tracking expenses, or handling reimbursement records."
---

# i-rs-invoice

Invoice management CLI tool for tracking expense invoices and reimbursement status.

## Storage

- Config: `~/.config/i-rs/invoice.json`
- Can be overridden with `CONFIG_DIR` environment variable

## Data Model

```rust
struct Invoice {
    id: String,           // UUID
    name: String,        // Invoice name
    amount: f64,         // Invoice amount
    date: DateTime,      // Invoice date
    invoice_type: InvoiceType,  // electronic or paper
    reimbursed: bool,    // Reimbursement status
    tags: Vec<String>,   // Tags for categorization
    remark: Vec<String>, // Remarks/notes
    created_at: DateTime,
    updated_at: DateTime,
}

enum InvoiceType {
    Electronic,
    Paper,
}
```

## Commands

### add
Add a new invoice:
```bash
i-rs-invoice add <name> --amount <amount> [options]
```
Options:
- `--date, -d`: Invoice date (YYYY-MM-DD)
- `--type, -t`: Invoice type (electronic/paper)
- `--reimbursed, -r`: Mark as reimbursed
- `--tags`: Add tags
- `--remark`: Add remarks

### list
List all invoices:
```bash
i-rs-invoice list [options]
```
Options:
- `--tag`: Filter by tag
- `--reimbursed`: Show only reimbursed
- `--unreimbursed`: Show only unreimbursed

### get
Get invoice details:
```bash
i-rs-invoice get <id>
```

### update
Update an invoice:
```bash
i-rs-invoice update <id> [options]
```
Options:
- `--name`: Update name
- `--amount`: Update amount
- `--date`: Update date
- `--type`: Update invoice type
- `--reimbursed`: Update reimbursement status
- `--add-tags`: Add tags
- `--remove-tags`: Remove tags
- `--add-remark`: Add remarks

### delete
Delete an invoice:
```bash
i-rs-invoice delete <id>
```

### stats
View invoice statistics:
```bash
i-rs-invoice stats [options]
```
Options:
- `--tag`: Filter by tag
- `--reimbursed`: Show only reimbursed stats
- `--unreimbursed`: Show only unreimbursed stats

### example
Show usage examples:
```bash
i-rs-invoice example
```

## Global Flags

- `--json`: Output in JSON format

### data

Manage data (export, import, clear).

```bash
i-rs-invoice data export
i-rs-invoice data import [FILE]
i-rs-invoice data clear
```

### skill

Show skill information.

```bash
i-rs-invoice skill [summary|content|raw]
```

## Examples

```bash
# Add electronic invoice
i-rs-invoice add "Office Supplies" --amount 299.99 --type electronic --tags expense [OPTIONS]

# Add paper invoice with reimbursement
i-rs-invoice add "Travel" --amount 1500.00 --type paper --reimbursed --tags travel [OPTIONS]

# List unreimbursed invoices
i-rs-invoice list --unreimbursed

# View statistics
i-rs-invoice stats --tag expense
```
