# i-rs-podcast Test Records

## Setup

```bash
# Build
cargo build -p i-rs-podcast
```

## Test Commands

### 1. Add Podcasts

```bash
# Add podcast 1
cargo run -p i-rs-podcast -- add "Test Podcast 1" --author "Author A" --duration 3600 --tag test --tag podcast

# Add podcast 2
cargo run -p i-rs-podcast -- add "Test Podcast 2" --author "Author B" --duration 7200 --tag test --tag course

# Add podcast 3
cargo run -p i-rs-podcast -- add "Test Podcast 3" --author "Author C" --duration 1800 --tag test
```

### 2. List Podcasts

```bash
# List all
cargo run -p i-rs-podcast -- list

# List by status
cargo run -p i-rs-podcast -- list --status not_started

# List by tag
cargo run -p i-rs-podcast -- list --tag test
```

### 3. Update Progress

```bash
# Update position (30 min)
cargo run -p i-rs-podcast -- listen "Test Podcast 1" --position 1800

# Update with notes
cargo run -p i-rs-podcast -- listen "Test Podcast 1" --position 1800 --notes "First half completed"

# Complete podcast
cargo run -p i-rs-podcast -- listen "Test Podcast 3" --position 1800
```

### 4. Get Details

```bash
cargo run -p i-rs-podcast -- get "Test Podcast 1"
cargo run -p i-rs-podcast -- get "Test Podcast 2"
```

### 5. Update Podcast

```bash
cargo run -p i-rs-podcast -- update "Test Podcast 2" --author "Updated Author"
cargo run -p i-rs-podcast -- update "Test Podcast 2" --tag course --tag tutorial
```

### 6. Statistics

```bash
cargo run -p i-rs-podcast -- stats
```

### 7. JSON Output

```bash
cargo run -p i-rs-podcast -- list --json
cargo run -p i-rs-podcast -- get "Test Podcast 1" --json
cargo run -p i-rs-podcast -- stats --json
```

### 8. Example and Skill

```bash
cargo run -p i-rs-podcast -- example
cargo run -p i-rs-podcast -- skill
cargo run -p i-rs-podcast -- skill summary
```

### 9. Delete

```bash
cargo run -p i-rs-podcast -- delete "Test Podcast 3"
cargo run -p i-rs-podcast -- list
```

## Cleanup

```bash
# Remove test data
rm ~/.config/i-rs/podcasts.json
```
