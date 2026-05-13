# i-rs-weight

Weight tracking CLI tool for managing weight records with trend visualization.

## Install

```bash
npm install -g @i-rs/i-rs-weight
# or
brew install i-rs/homebrew-tap/i-rs-weight
```

## Usage

### Add Weight Record

```bash
i-rs-weight add <DATE> <WEIGHT> [OPTIONS]
```

Options:
- `-r, --remark <REMARK>` - Remarks (can be specified multiple times)

### List Weight Records

```bash
i-rs-weight list [OPTIONS]
```

Options:
- `-d, --days <DAYS>` - Show records from last N days
- `-c, --chart` - Show ASCII trend chart
- `-s, --stats` - Show statistics (min/max/avg/change)

### Update Weight Record

```bash
i-rs-weight update <DATE> [OPTIONS]
```

Options:
- `-w, --weight <WEIGHT>` - New weight value
- `-r, --remark <REMARK>` - New remarks

### Delete Weight Record

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

# Show only chart
i-rs-weight list --chart

# Show only stats
i-rs-weight list --stats

# Update a record
i-rs-weight update 2025-01-15 --weight 70.0

# Delete a record
i-rs-weight delete 2025-01-15
```

## Chart Example

```
Weight Trend (Last 30 days)
─────────────────────────────────────
 71.0 ●
 70.5    │
 70.0 ●──│──●
 69.5    │
 69.0 ●────────────────────●
 68.5
 68.0

 01-15               01-20

  70.5 → 68.0 (-2.5 kg ↓)
```

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/weights.json`
- Linux: `~/.config/i-rs/weights.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-weight list
```

## License

MIT OR Apache-2.0
