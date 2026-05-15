---
name: "i-rs-tax"
description: "Tax record management CLI tool. Record personal income tax, VAT, and other tax information with annual statistics and filing status tracking."
---

# i-rs-tax

## Global Flags

- `--json` — Output in JSON format

Tax record management CLI tool for recording personal income tax, VAT, and other tax information with annual statistics and filing status tracking.

## Storage

- Config: `~/.config/i-rs/tax.json`
- Environment variable `CONFIG_DIR` overrides default path

## Tax Types

- `personal`: Personal income tax
- `vat`: Value-added tax

## Filing Status

- `unreported`: Not yet filed
- `filing`: Filing in progress
- `filed`: Already filed
- `paid`: Already paid

## Commands

### add - Add record
```bash
i-rs-tax add <NAME> --tax-type <TYPE> --amount <AMOUNT> --date <DATE> [OPTIONS]
```

Options:
- `--tax-type, -t`: Tax type (personal/vat)
- `--amount, -a`: Amount
- `--date, -d`: Date (YYYY-MM-DD)
- `--year, -y`: Tax year (optional, defaults from date)
- `--status, -s`: Filing status (optional, default unreported)
- `--tag`: Tags (repeatable)
- `--remark`: Remarks (repeatable)

Example:
```bash
i-rs-tax add Income2024 -t personal -a 12000 -d 2024-03-15 -s filed --tag salary
```

### list - List records
```bash
i-rs-tax list [OPTIONS]
```

Options:
- `--tag, -t`: Filter by tag
- `--year, -y`: Filter by year
- `--tax-type`: Filter by tax type (personal/vat)

### get - View details
```bash
i-rs-tax get <NAME>
```

### delete - Delete record
```bash
i-rs-tax delete <NAME>
```

### stats - Annual statistics
```bash
i-rs-tax stats [OPTIONS]
```

Options:
- `--year, -y`: Specific year (defaults to current year)
- `--tax-type, -t`: Filter by tax type

### example - Usage examples
```bash
i-rs-tax example
```

### skill - AI skill documentation
```bash
i-rs-tax skill [summary]
```

## Data Structure

```json
{
  "name": "Income2024",
  "tax_type": "personal",
  "amount": 12000.00,
  "date": "2024-03-15",
  "year": 2024,
  "status": "filed",
  "tags": ["salary"],
  "remark": [],
  "created_at": "2024-03-15T10:00:00Z",
  "updated_at": "2024-03-15T10:00:00Z"
}
```

## Typical Use Cases

1. **Monthly personal income tax**: Record monthly salary tax, track filing status
2. **Quarterly VAT filing**: Record quarterly VAT for annual summary
3. **Year-end bonus tax**: Separately track year-end bonus tax info
4. **Annual summary**: Use stats command for yearly tax overview

### data - Data management

Manage data (export, import, clear).

```bash
i-rs-tax data export
i-rs-tax data import [FILE]
i-rs-tax data clear
```
