# i-rs-article Examples

## Basic Usage

### Add Articles

```bash
# Basic article
i-rs-article add rust-blog https://rust-lang.org/blog "Rust Blog" --source rust-lang.org

# With tags
i-rs-article add python-guide https://docs.python.org "Python Guide" --source python.org --tag programming --tag tutorial

# With remarks
i-rs-article add golang-article https://go.dev/blog "Go Blog" --source go.dev --tag programming --remark "Important reading"
```

### List Articles

```bash
# List all
i-rs-article list

# Filter by tag
i-rs-article list --tag programming

# Filter by status
i-rs-article list --status unread
i-rs-article list --status reading
i-rs-article list --status read

# Combined filter
i-rs-article list --tag programming --status unread
```

### Reading Workflow

```bash
# Start reading
i-rs-article update rust-blog --status reading

# Add notes while reading
i-rs-article update rust-blog --notes "Rust ownership is interesting" "Memory safety without GC"

# Mark as read
i-rs-article read rust-blog
```

### Get Article Details

```bash
i-rs-article get rust-blog
```

### Update Articles

```bash
# Update status
i-rs-article update rust-blog --status reading
i-rs-article update rust-blog --status read

# Update tags
i-rs-article update rust-blog --tag rust --tag tutorial

# Add remarks
i-rs-article update rust-blog --remark "Must read carefully"

# Add notes
i-rs-article update rust-blog --notes "Key insight about lifetimes"
```

### Delete Articles

```bash
i-rs-article delete old-article
```

### View Statistics

```bash
i-rs-article stats
```

## JSON Output

```bash
# List with JSON
i-rs-article list --json

# Get with JSON
i-rs-article get rust-blog --json
```
