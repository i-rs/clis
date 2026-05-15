# i-rs-debt

Debt management CLI tool for tracking credit card debts, loans, and borrowed money with payment history and overdue reminders.

## Features

- Multiple debt types: credit card, loan, borrowed
- Payment history tracking
- Installment payment tracking
- Overdue reminders
- Tag support
- Statistics view
- JSON output support

## Install

```bash
npm install -g @i-rs/i-rs-debt
# or
brew install i-rs/homebrew-tap/i-rs-debt
```

## Quick Start

```bash
# Add credit card debt
i-rs-debt add "Credit Card A" --debt-type credit_card --amount 10000 --interest-rate 15.0

# Add a loan
i-rs-debt add "Car Loan" --debt-type loan --amount 50000 --tags car,vehicle

# List all debts
i-rs-debt list

# Record a payment
i-rs-debt pay "Credit Card A" --amount 500

# View debt details
i-rs-debt get "Credit Card A"

# View statistics
i-rs-debt stats
```

## Data Storage

- macOS: `~/.config/i-rs/debt.json`
- Linux: `~/.config/i-rs/debt.json`
- Windows: `~\AppData\Roaming\i-rs\debt.json`

## License

MIT OR Apache-2.0
