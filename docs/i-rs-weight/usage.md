# i-rs-weight Usage Guide

## Install

```bash
npm install -g @i-rs/i-rs-weight
# or
brew install i-rs/homebrew-tap/i-rs-weight
```

## Commands

### add

Add a weight record.

```bash
i-rs-weight add <DATE> <WEIGHT>
```

Options:
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List weight records.

```bash
i-rs-weight list [OPTIONS]
```

Options:
- `-d, --days <DAYS>` - Show records from last N days
- `-c, --chart` - Show ASCII trend chart
- `-s, --stats` - Show statistics

### update

Update a weight record.

```bash
i-rs-weight update <DATE>
```

Options:
- `-w, --weight <WEIGHT>` - New weight value
- `-r, --remark <REMARK>` - New remarks

### delete

Delete a weight record.

```bash
i-rs-weight delete <DATE>
```

## Examples

```bash
# Add a weight record
i-rs-weight add 2025-01-15 70.5

# Add with remarks
i-rs-weight add 2025-01-16 70.3 --remark "After workout"

# List all records
i-rs-weight list

# List last 30 days with chart and stats
i-rs-weight list --days 30 --chart --stats

# Update a record
i-rs-weight update 2025-01-15 --weight 70.0

# Delete a record
i-rs-weight delete 2025-01-15
```

## Data Storage

- macOS: `~/.config/i-rs/weights.json`
- Linux: `~/.config/i-rs/weights.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`
