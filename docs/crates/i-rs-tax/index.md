# i-rs-tax Tax Record Management

Tax record management CLI tool for recording personal income tax, VAT, and other tax information with annual statistics and filing status tracking.

## Features

- **Tax Record Management**: Support for personal income tax and VAT
- **Amount Tracking**: Record tax amounts
- **Date Management**: Record tax dates with automatic year extraction
- **Filing Status**: Support unreported, filing, filed, paid statuses
- **Tag System**: Organize tax records with tags
- **Annual Statistics**: View yearly tax summaries

## Quick Start

```bash
# Add personal income tax
i-rs-tax add Income2024 --tax-type personal --amount 12000 --date 2024-03-15

# List all records
i-rs-tax list

# Filter by year
i-rs-tax list --year 2024

# View annual statistics
i-rs-tax stats --year 2024
```

## Data Storage

- macOS: `~/.config/i-rs/tax.json`
- Linux: `~/.config/i-rs/tax.json`
- Windows: `~\AppData\Roaming\i-rs\tax.json`

## License

MIT OR Apache-2.0
