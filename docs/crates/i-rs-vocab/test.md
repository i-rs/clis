# i-rs-vocab Test Records

## Test Commands

### Add Words

```bash
# Test basic add
i-rs-vocab add testword "a procedure for testing" --tag testing

# Test add with examples
i-rs-vocab add serendipity "fortunate accident" --tag vocabulary --example "It was pure serendipity."

# Test add with status
i-rs-vocab add learning "in the process of learning" --tag testing --status learning
```

### List Words

```bash
# List all
i-rs-vocab list

# List by status
i-rs-vocab list --status new
i-rs-vocab list --status learning
i-rs-vocab list --status mastered

# List by tag
i-rs-vocab list --tag testing
i-rs-vocab list --tag vocabulary
```

### Get Word

```bash
i-rs-vocab get testword
i-rs-vocab get serendipity
```

### Update Word

```bash
# Update definition
i-rs-vocab update testword --definition "a test procedure"

# Update status
i-rs-vocab update testword --status learning
i-rs-vocab update testword --status mastered

# Record review
i-rs-vocab update testword --review
```

### Delete Word

```bash
i-rs-vocab delete testword
```

### Quiz

```bash
i-rs-vocab quiz
i-rs-vocab quiz --count 5
```

### Stats

```bash
i-rs-vocab stats
```

## Expected Output

### List Output
```
┌──────────────────┬─────────────────────┬────────────┬─────────┬────────────┐
│ WORD             │ DEFINITION          │ STATUS    │ REVIEWS │ TAGS       │
├──────────────────┼─────────────────────┼───────────┼─────────┼────────────┤
│ serendipity      │ fortunate accident  │ New 🆕    │ 0       │ vocabulary │
│ learning         │ in the process...   │ Learning  │ 0       │ testing    │
└──────────────────┴─────────────────────┴───────────┴─────────┴────────────┘

Total: 2 words

Statistics:
  Total: 2
  New: 1 (🆕)
  Learning: 1 (📖)
  Mastered: 0 (✅)
  Reviews: 0
```

### Stats Output
```
Vocabulary Statistics

Overall:
  Total words:    10
  Total reviews:  25

By Status:
  New:            3 (🆕)
  Learning:       4 (📖)
  Mastered:       3 (✅)

Progress:
  [████████░░░░░░░░░░░░░░░░] 30.0% mastered
```
