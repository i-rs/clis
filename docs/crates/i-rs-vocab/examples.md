# i-rs-vocab Examples

## Basic Usage

### Adding Words

```bash
# Add a simple word
i-rs-vocab add hello "greeting; hello world"

# Add with tags
i-rs-vocab add serendipity "the occurrence of events by chance in a happy way" --tag vocabulary --tag life

# Add with examples
i-rs-vocab add ephemeral "lasting for a very short time" \
  --tag advanced \
  --example "Fame is ephemeral." \
  --example "The beauty of cherry blossoms is ephemeral."

# Add with multiple tags
i-rs-vocab add ubiquitous "present, appearing, or found everywhere" \
  --tag vocabulary \
  --tag advanced \
  --tag academic
```

### Listing Words

```bash
# List all words
i-rs-vocab list

# List new words only
i-rs-vocab list --status new

# List learning words only
i-rs-vocab list --status learning

# List mastered words only
i-rs-vocab list --status mastered

# Filter by tag
i-rs-vocab list --tag basic
i-rs-vocab list --tag advanced
```

### Getting Word Details

```bash
# Get word details
i-rs-vocab get hello

# View full word information including examples and remarks
i-rs-vocab get serendipity
```

### Updating Words

```bash
# Update definition
i-rs-vocab update hello --definition "a greeting or expression of goodwill"

# Update status
i-rs-vocab update hello --status learning
i-rs-vocab update hello --status mastered

# Add new example
i-rs-vocab update hello --example "Hello, nice to meet you!"

# Record a review
i-rs-vocab update hello --review

# Update tags
i-rs-vocab update hello --tag greetings --tag basics
```

### Deleting Words

```bash
# Delete a word
i-rs-vocab delete hello
```

## Quiz Mode

```bash
# Start a quiz with default 5 words
i-rs-vocab quiz

# Quiz 10 words
i-rs-vocab quiz --count 10

# Quiz 20 words
i-rs-vocab quiz -c 20
```

Quiz mode will:
1. Show you words to review
2. Prompt you to enter the definition
3. Check your answer
4. Track your progress
5. Update word status based on review count

## Statistics

```bash
# View learning statistics
i-rs-vocab stats
```

Shows:
- Total number of words
- Words by status (new/learning/mastered)
- Total review count
- Progress bar showing mastery percentage

## JSON Output

```bash
# List words in JSON
i-rs-vocab list --json

# Get word in JSON
i-rs-vocab get hello --json

# Filter and output JSON
i-rs-vocab list --status learning --json
```

## Learning Workflow

1. **Add new words**:
   ```bash
   i-rs-vocab add newword "definition" --tag topic
   ```

2. **Review regularly**:
   ```bash
   i-rs-vocab quiz
   ```

3. **Update status**:
   ```bash
   i-rs-vocab update newword --review
   ```

4. **Track progress**:
   ```bash
   i-rs-vocab stats
   ```

## Mastery Progression

- Words start as `new`
- After 2+ reviews: status becomes `learning`
- After 5+ reviews: status becomes `mastered`

## Environment Variables

| Variable | Description |
|----------|-------------|
| `CONFIG_DIR` | Override config directory |
