# i-rs-event Usage

## Commands

### add

Add a new event.

```bash
i-rs-event add <name> [OPTIONS]

OPTIONS:
  -d, --date <DATE>           Event date (YYYY-MM-DD or YYYY-MM-DD HH:MM)
  -t, --type <TYPE>           Event type (meeting, gathering, course, other)
  -l, --location <LOCATION>  Event location
  -p, --participants <P>      Participants (comma-separated)
  -g, --tags <TAGS>           Tags (comma-separated)
  -r, --remark <REMARK>       Remarks (can be specified multiple times)
  --json                      Output JSON format
```

### list

List all events with optional filters.

```bash
i-rs-event list [OPTIONS]

OPTIONS:
  -t, --tag <TAG>        Filter by tag
  -y, --type <TYPE>       Filter by event type
  --json                  Output JSON format
```

### get

Get detailed information about an event.

```bash
i-rs-event get <name> [OPTIONS]

OPTIONS:
  --json    Output JSON format
```

### delete

Delete an event.

```bash
i-rs-event delete <name> [OPTIONS]

OPTIONS:
  --json    Output JSON format
```

### stats

View event statistics.

```bash
i-rs-event stats [OPTIONS]

OPTIONS:
  -y, --year <YEAR>    Year for statistics (default: current year)
  --json               Output JSON format
```

### example

Show usage examples.

```bash
i-rs-event example
```

### skill

Show AI skill documentation.

```bash
i-rs-event skill [summary|content]
```
