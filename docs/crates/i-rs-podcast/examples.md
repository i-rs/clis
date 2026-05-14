# i-rs-podcast Examples

## Basic Usage

### Add a Podcast

```bash
# Basic podcast
i-rs-podcast add "The Daily"

# With author and duration (duration in seconds)
i-rs-podcast add "The Daily" --author "New York Times" --duration 3600

# With tags
i-rs-podcast add "Rust Course" --author "Ferris" --duration 7200 --tag rust --tag programming

# With remarks
i-rs-podcast add "Tech News Weekly" --author "Molly Wood" --duration 2700 --remark "Morning commute podcast"
```

### List Podcasts

```bash
# List all
i-rs-podcast list

# Filter by status
i-rs-podcast list --status not_started
i-rs-podcast list --status in_progress
i-rs-podcast list --status completed

# Filter by tag
i-rs-podcast list --tag programming
i-rs-podcast list --tag rust
```

### Get Podcast Details

```bash
i-rs-podcast get "The Daily"
i-rs-podcast get "Rust Course"
```

### Update Progress

```bash
# Update position (30 minutes = 1800 seconds)
i-rs-podcast listen "The Daily" --position 1800

# Update position with notes
i-rs-podcast listen "Rust Course" --position 3600 --notes "Understanding ownership"
i-rs-podcast listen "Rust Course" --position 7200 --notes "Chapter 5: Structs" --notes "Key takeaway: methods"

# Mark as completed
i-rs-podcast listen "The Daily" --position 3600
```

### Update Podcast Info

```bash
# Update author
i-rs-podcast update "The Daily" --author "NYT"

# Update duration
i-rs-podcast update "Rust Course" --duration 10800

# Add tags
i-rs-podcast update "Rust Course" --tag tutorial --tag beginner

# Add remarks
i-rs-podcast update "Tech News Weekly" --remark "Best tech podcast"
```

### Delete Podcast

```bash
i-rs-podcast delete "Old Podcast"
```

### View Statistics

```bash
i-rs-podcast stats
```

## Advanced Usage

### Course Tracking

```bash
# Add a course
i-rs-podcast add "Advanced Rust" --author "Rust Team" --duration 14400 --tag rust --tag advanced

# Track progress per section
i-rs-podcast listen "Advanced Rust" --position 1800 --notes "Section 1: Lifetimes"
i-rs-podcast listen "Advanced Rust" --position 3600 --notes "Section 2: Generics"
i-rs-podcast listen "Advanced Rust" --position 5400 --notes "Section 3: Traits"

# Complete course
i-rs-podcast listen "Advanced Rust" --position 14400
```

### Podcast Series Tracking

```bash
# Add multiple episodes
i-rs-podcast add "S01E01: Introduction" --author "Host" --duration 1800 --tag series
i-rs-podcast add "S01E02: Getting Started" --author "Host" --duration 2100 --tag series
i-rs-podcast add "S01E03: Advanced Topics" --author "Host" --duration 2400 --tag series

# Track current episode
i-rs-podcast listen "S01E01: Introduction" --position 1800
i-rs-podcast listen "S01E02: Getting Started" --position 1050

# View in-progress series
i-rs-podcast list --tag series --status in_progress
```

### JSON Output

```bash
# List in JSON
i-rs-podcast list --json

# Get details in JSON
i-rs-podcast get "The Daily" --json

# Stats in JSON
i-rs-podcast stats --json
```

## Duration Format

Duration is specified in seconds:
- 60 seconds = 1 minute
- 600 seconds = 10 minutes
- 1800 seconds = 30 minutes
- 3600 seconds = 1 hour
- 7200 seconds = 2 hours

Common conversions:
| Duration | Seconds |
|---------|--------|
| 5 min | 300 |
| 15 min | 900 |
| 30 min | 1800 |
| 45 min | 2700 |
| 1 hour | 3600 |
| 2 hours | 7200 |
