---
title: i-rs-vocab
description: Vocabulary learning CLI tool for managing and reviewing vocabulary words.
---

# i-rs-vocab

Vocabulary learning CLI tool for managing and reviewing vocabulary words with spaced repetition.

## Features

- Add vocabulary words with definitions and examples
- Track learning status (new/learning/mastered)
- Quiz mode for active recall practice
- Statistics showing learning progress
- Tag support for organization
- Review count tracking for mastery progression

## Quick Start

```bash
# Add a new word
i-rs-vocab add hello "greeting" --tag basic --example "Hello, how are you?"

# List all words
i-rs-vocab list

# Start a quiz
i-rs-vocab quiz

# View statistics
i-rs-vocab stats
```

## Data Storage

- Config: `~/.config/i-rs/vocab.json`
