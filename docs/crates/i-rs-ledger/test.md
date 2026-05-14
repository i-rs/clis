# i-rs-ledger Test Records

## Test Data

```bash
# Income
i-rs-ledger add --type income --amount 5000 --category salary --date 2024-01-01
i-rs-ledger add --type income --amount 1500 --category freelance --date 2024-01-15

# Expenses
i-rs-ledger add --type expense --amount 50 --category food --date 2024-01-05
i-rs-ledger add --type expense --amount 150 --category groceries --date 2024-01-10
i-rs-ledger add --type expense --amount 30 --category transport --date 2024-01-12

# Transfers
i-rs-ledger add --type transfer --amount 1000 --category savings --date 2024-01-01
```