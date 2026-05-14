# i-rs-article Test Records

## Test Commands

### Add Articles

```bash
# Add test article 1
i-rs-article add test-article-1 https://example.com/article1 "Test Article 1" --source example.com --tag test --tag article

# Add test article 2
i-rs-article add test-article-2 https://example.com/article2 "Test Article 2" --source example.com --tag test

# Add test article 3
i-rs-article add test-article-3 https://example.com/article3 "Test Article 3" --source example.com --tag test
```

### List Articles

```bash
# List all
i-rs-article list

# List with tag filter
i-rs-article list --tag test

# List with status filter
i-rs-article list --status unread
```

### Get Article

```bash
i-rs-article get test-article-1
```

### Update Articles

```bash
# Update status to reading
i-rs-article update test-article-1 --status reading

# Add notes
i-rs-article update test-article-1 --notes "Test note 1" "Test note 2"
```

### Read Article

```bash
i-rs-article read test-article-2
```

### View Stats

```bash
i-rs-article stats
```

### Delete Articles

```bash
i-rs-article delete test-article-1
i-rs-article delete test-article-2
i-rs-article delete test-article-3
```

## Expected Results

- Add: Success message with "✓ Article added successfully"
- List: Table with article details
- Get: Article detail view with all fields
- Read: Success message showing status change
- Update: Success message with "✓ Article updated successfully"
- Delete: Success message with "✓ Article deleted successfully"
- Stats: Statistics showing total, unread, reading, read counts
