# i-rs-tax

Tax record management CLI tool for recording personal income tax, VAT, and other tax information.

## Features

- Tax record management (personal income tax / VAT)
- Amount and date tracking
- Filing status management
- Tag-based categorization
- Annual statistics
- JSON output support

## Install

```bash
cargo install i-rs-tax
# or
brew install i-rs/homebrew-tap/i-rs-tax
```

## Quick Start

### Add Tax Record

```bash
# Add personal income tax
i-rs-tax add Income2024 --tax-type personal --amount 12000 --date 2024-03-15

# Add VAT record
i-rs-tax add VAT-Q1 --tax-type vat --amount 5000 --date 2024-04-01 --status filed
```

### View Records

```bash
# List all records
i-rs-tax list

# Filter by year
i-rs-tax list --year 2024

# Filter by tag
i-rs-tax list --tag salary

# Filter by tax type
i-rs-tax list --tax-type personal
```

### View Details

```bash
i-rs-tax get Income2024
```

### Delete Record

```bash
i-rs-tax delete Income2024
```

### Annual Statistics

```bash
# Current year stats
i-rs-tax stats

# Specific year stats
i-rs-tax stats --year 2024
```

## Tax Types

- `personal`: Personal income tax
- `vat`: Value-added tax

## Filing Status

- `unreported`: Not yet filed
- `filing`: Filing in progress
- `filed`: Already filed
- `paid`: Already paid

## Data Storage

- macOS: `~/.config/i-rs/tax.json`
- Linux: `~/.config/i-rs/tax.json`
- Windows: `~\AppData\Roaming\i-rs\tax.json`

Override with `CONFIG_DIR` environment variable.

## JSON Output

All commands support `--json` flag:

```bash
i-rs-tax list --json
i-rs-tax get Income2024 --json
i-rs-tax stats --year 2024 --json
```

## Examples

```bash
# Add a complete tax record
i-rs-tax add Income2024 \
  --tax-type personal \
  --amount 12000 \
  --date 2024-03-15 \
  --status filed \
  --tag salary \
  --tag bonus \
  --remark Year-end bonus declaration

# View all personal income tax records
i-rs-tax list --tax-type personal

# View 2024 annual tax statistics
i-rs-tax stats --year 2024
```

## License

MIT OR Apache-2.0
