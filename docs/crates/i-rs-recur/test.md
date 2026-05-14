# i-rs-recur Test Records

## Test Data

```bash
# Monthly subscriptions
i-rs-recur add "Netflix" --amount 15.99 --frequency monthly
i-rs-recur add "Spotify" --amount 9.99 --frequency monthly
i-rs-recur add "iCloud" --amount 6.99 --frequency monthly

# Monthly bills
i-rs-recur add "Rent" --amount 2000 --frequency monthly
i-rs-recur add "Internet" --amount 80 --frequency monthly
i-rs-recur add "Phone" --amount 50 --frequency monthly

# Quarterly
i-rs-recur add "Amazon Prime" --amount 99 --frequency quarterly

# Yearly
i-rs-recur add "GitHub Pro" --amount 48 --frequency yearly
i-rs-recur add "Domain" --amount 10 --frequency yearly --tag website
```