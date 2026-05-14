# i-rs-time Usage

## Commands

### start

Start a new timer.

```bash
i-rs-time start "Task name"
i-rs-time start "Meeting" --tag work
i-rs-time start "Coding" --tag development --remark "Feature implementation"
```

Options:
- `--tag, -t` - Add tags to the entry
- `--remark, -r` - Add remarks to the entry

### stop

Stop the current timer.

```bash
i-rs-time stop
```

### list

List all time entries.

```bash
i-rs-time list
i-rs-time list --tag work
```

Options:
- `--tag, -t` - Filter by tag

### stats

Show statistics for a period.

```bash
i-rs-time stats today
i-rs-time stats yesterday
i-rs-time stats week
```

Periods: `today`, `yesterday`, `week`

### report

Generate work reports.

```bash
i-rs-time report --days 7
i-rs-time report --start 2024-01-01 --end 2024-01-31
```

Options:
- `--days` - Number of days to report
- `--start` - Start date (YYYY-MM-DD)
- `--end` - End date (YYYY-MM-DD)

### get

Get entry details by ID.

```bash
i-rs-time get <entry-id>
```

### delete

Delete an entry by ID.

```bash
i-rs-time delete <entry-id>
```

### example

Show usage examples.

```bash
i-rs-time example
```

### skill

View AI skill documentation.

```bash
i-rs-time skill
i-rs-time skill summary
i-rs-time skill content
i-rs-time skill raw
```

## Global Options

- `--json, -j` - Output in JSON format
