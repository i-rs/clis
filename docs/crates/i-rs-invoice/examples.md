# i-rs-invoice Examples

## Add Invoices

### Electronic Invoice

```bash
i-rs-invoice add "Office Supplies" --amount 299.99 --type electronic --tags expense --tags office
```

### Paper Invoice

```bash
i-rs-invoice add "Travel Expense" --amount 1500.00 --date 2024-01-15 --type paper --tags travel
```

### Invoice with Multiple Tags

```bash
i-rs-invoice add "Client Lunch" --amount 258.00 --type electronic --tags business --tags meal --tags client
```

### Reimbursed Invoice

```bash
i-rs-invoice add "Conference Registration" --amount 2000.00 --type electronic --reimbursed --tags business --tags education
```

## List Invoices

### List All

```bash
i-rs-invoice list
```

### List by Tag

```bash
i-rs-invoice list --tag expense
i-rs-invoice list --tag travel
```

### List Reimbursed

```bash
i-rs-invoice list --reimbursed
```

### List Unreimbursed

```bash
i-rs-invoice list --unreimbursed
```

## Get Invoice Details

```bash
i-rs-invoice get <invoice-id>
```

## Update Invoices

### Update Name

```bash
i-rs-invoice update <id> --name "Updated Invoice Name"
```

### Mark as Reimbursed

```bash
i-rs-invoice update <id> --reimbursed
```

### Add Tags

```bash
i-rs-invoice update <id> --add-tags business --add-tags approved
```

### Remove Tags

```bash
i-rs-invoice update <id> --remove-tags pending
```

### Update Amount

```bash
i-rs-invoice update <id> --amount 350.00
```

## Statistics

### Overall Statistics

```bash
i-rs-invoice stats
```

### By Tag

```bash
i-rs-invoice stats --tag expense
```

### Reimbursed Statistics

```bash
i-rs-invoice stats --reimbursed
```

### Unreimbursed Statistics

```bash
i-rs-invoice stats --unreimbursed
```

## Delete Invoice

```bash
i-rs-invoice delete <invoice-id>
```

## JSON Output

All commands support `--json` flag:

```bash
i-rs-invoice list --json
i-rs-invoice get <id> --json
i-rs-invoice add "Test" --amount 100 --json
```
