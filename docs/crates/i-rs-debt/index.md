# i-rs-debt Usage Guide

## add - Add Debt

Create a new debt record.

```bash
i-rs-debt add <name> --debt-type <type> --amount <amount> [options]
```

**Required:**
- `name`: Debt name
- `--debt-type`: Debt type (credit_card, loan, borrowed)
- `--amount`: Total debt amount

**Options:**
- `--interest-rate`: Annual interest rate (percentage)
- `--due-date`: Due date (YYYY-MM-DD)
- `--tags`: Tags (comma separated)
- `--remark`: Remarks (repeatable)

**Examples:**
```bash
i-rs-debt add "Credit Card A" --debt-type credit_card --amount 10000 --interest-rate 15.0
i-rs-debt add "Car Loan" --debt-type loan --amount 50000 --tags car,vehicle --due-date 2026-12-31
```

---

## list - List Debts

List all debts with optional filtering.

```bash
i-rs-debt list [options]
```

**Options:**
- `--tag`: Filter by tag
- `--overdue`: Show only overdue debts
- `--paid-off`: Show only paid-off debts

**Examples:**
```bash
i-rs-debt list
i-rs-debt list --tag car
i-rs-debt list --overdue
```

---

## get - View Debt Details

View detailed information for a single debt.

```bash
i-rs-debt get <name> [options]
```

**Options:**
- `--payments`: Show payment history

**Examples:**
```bash
i-rs-debt get "Credit Card A"
i-rs-debt get "Credit Card A" --payments
```

---

## pay - Record Payment

Record a payment.

```bash
i-rs-debt pay <name> --amount <amount> [options]
```

**Required:**
- `name`: Debt name
- `--amount`: Payment amount

**Options:**
- `--note`: Payment note

**Examples:**
```bash
i-rs-debt pay "Credit Card A" --amount 500 --note "Monthly payment"
```

---

## update - Update Debt

Update debt information.

```bash
i-rs-debt update <name> [options]
```

**Options:**
- `--rename`: New name
- `--debt-type`: New debt type
- `--amount`: New total amount
- `--interest-rate`: New interest rate
- `--due-date`: New due date
- `--add-tags`: Add tags
- `--remove-tags`: Remove tags
- `--add-remark`: Add remarks

**Examples:**
```bash
i-rs-debt update "Credit Card A" --interest-rate 12.0
i-rs-debt update "Credit Card A" --add-tags important
```

---

## delete - Delete Debt

Delete a debt record.

```bash
i-rs-debt delete <name> [options]
```

**Options:**
- `--force`: Skip confirmation

**Examples:**
```bash
i-rs-debt delete "Old Debt" --force
```

---

## stats - Statistics

Show debt statistics.

```bash
i-rs-debt stats [options]
```

**Options:**
- `--by-type`: Group by debt type

**Examples:**
```bash
i-rs-debt stats
i-rs-debt stats --by-type
```

---

## Global Options

- `--json`: Output in JSON format

---

## example - Usage Examples

Show detailed usage examples.

```bash
i-rs-debt example
```

---

## skill - AI Skill Documentation

View AI skill documentation.

```bash
i-rs-debt skill      # Show full document
i-rs-debt skill summary  # Show summary
```
