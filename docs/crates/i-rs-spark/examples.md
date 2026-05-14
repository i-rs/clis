# i-rs-spark Examples

## Basic Usage

### Capturing Ideas

```bash
# Quick capture
i-rs-spark add "Use machine learning for text classification"
i-rs-spark add "Build a habit tracker app"
i-rs-spark add "Create an AI writing assistant"

# With sources
i-rs-spark add "New app idea" --source "Dream"
i-rs-spark add "Design pattern" --source "Book: Clean Code"
i-rs-spark add "API design" --source "Podcast: Software Engineering"
```

### With Tags

```bash
# Tagged ideas
i-rs-spark add "Habit tracking algorithm" --tag tech --tag app --tag health
i-rs-spark add "Marketing strategy" --tag business --tag marketing
i-rs-spark add "New side project" --tag project --tag creative
```

## Viewing Sparks

```bash
# List all
i-rs-spark list

# Filter by tag
i-rs-spark list --tag tech
i-rs-spark list --tag project

# Get details
i-rs-spark get abc12345
```

## Inspiration Sources

```bash
# From dreams
i-rs-spark add "Lucid dreaming app concept" --source "Dream"

# From reading
i-rs-spark add "Microservices architecture" --source "Book: Building Microservices"
i-rs-spark add "Note-taking method" --source "Article: Productivity"

# From conversations
i-rs-spark add "Team collaboration tool" --source "Conversation with colleague"
i-rs-spark add "API versioning strategy" --source "Discussion at meetup"

# From nature
i-rs-spark add "Swarm intelligence algorithm" --source "Observation: Ant colony"
```

## Idea Management

```bash
# Capture quickly
i-rs-spark add "Remember to research this later"
i-rs-spark add "Could be a good blog post topic"

# With remarks
i-rs-spark add "Start using this technique" --remark "Priority: high"
i-rs-spark add "Research further" --remark "Needs more investigation"
```