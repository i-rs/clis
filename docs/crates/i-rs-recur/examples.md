# i-rs-recur Examples

## Basic Usage

### Adding Subscriptions

```bash
# Monthly subscriptions
i-rs-recur add "Netflix" --amount 15.99 --frequency monthly
i-rs-recur add "Spotify" --amount 9.99 --frequency monthly
i-rs-recur add "iCloud" --amount 6.99 --frequency monthly

# Quarterly
i-rs-recur add "Amazon Prime" --amount 99 --frequency quarterly

# Yearly
i-rs-recur add "GitHub Pro" --amount 48 --frequency yearly
i-rs-recur add "Adobe CC" --amount 599 --frequency yearly
```

### Adding Bills

```bash
# Monthly bills
i-rs-recur add "Rent" --amount 2000 --frequency monthly
i-rs-recur add "Internet" --amount 80 --frequency monthly
i-rs-recur add "Phone" --amount 50 --frequency monthly

# Utilities (weekly for simpler tracking)
i-rs-recur add "Electricity" --amount 120 --frequency monthly
i-rs-recur add "Water" --amount 40 --frequency monthly
```

### With Tags

```bash
# Entertainment
i-rs-recur add "Netflix" --amount 15.99 --frequency monthly --tag entertainment --tag streaming
i-rs-recur add "Disney+" --amount 7.99 --frequency monthly --tag entertainment

# Essential
i-rs-recur add "Rent" --amount 2000 --frequency monthly --tag essential --tag housing
i-rs-recur add "Internet" --amount 80 --frequency monthly --tag essential --tag utilities
```

## Viewing Expenses

```bash
# List all
i-rs-recur list

# Get details
i-rs-recur get Netflix
```

## Managing Expenses

```bash
# Update amount
i-rs-recur update "Netflix" --amount 17.99

# Update frequency
i-rs-recur update "Spotify" --frequency yearly

# Delete
i-rs-recur delete "Old Service"
```

## Budget Planning

```bash
# Monthly essentials
i-rs-recur add "Rent" --amount 2000 --frequency monthly --tag essential
i-rs-recur add "Internet" --amount 80 --frequency monthly --tag essential
i-rs-recur add "Phone" --amount 50 --frequency monthly --tag essential
i-rs-recur add "Insurance" --amount 300 --frequency monthly --tag essential

# Entertainment budget
i-rs-recur add "Netflix" --amount 15.99 --frequency monthly --tag entertainment
i-rs-recur add "Spotify" --amount 9.99 --frequency monthly --tag entertainment
i-rs-recur add "Gaming" --amount 14.99 --frequency monthly --tag entertainment
```