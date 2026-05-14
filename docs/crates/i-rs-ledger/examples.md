# i-rs-ledger Examples

## Basic Usage

### Adding Income

```bash
# Salary
i-rs-ledger add --type income --amount 5000 --category salary

# Freelance
i-rs-ledger add --type income --amount 1500 --category freelance

# Investment returns
i-rs-ledger add --type income --amount 200 --category investment
```

### Adding Expenses

```bash
# Food
i-rs-ledger add --type expense --amount 50 --category food
i-rs-ledger add --type expense --amount 150 --category groceries

# Transportation
i-rs-ledger add --type expense --amount 30 --category transport

# Entertainment
i-rs-ledger add --type expense --amount 100 --category entertainment
```

### Adding Transfers

```bash
# To savings
i-rs-ledger add --type transfer --amount 1000 --category savings

# To investment
i-rs-ledger add --type transfer --amount 500 --category investment
```

## Filtering

```bash
# List only expenses
i-rs-ledger list --type expense

# List by category
i-rs-ledger list --category food
```

## With Tags

```bash
# Tagged expenses
i-rs-ledger add --type expense --amount 200 --category dining --tag restaurant --tag business
i-rs-ledger add --type expense --amount 80 --category transport --tag commute
```