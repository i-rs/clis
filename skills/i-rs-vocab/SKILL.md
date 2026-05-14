---
name: "i-rs-vocab"
description: "Vocabulary learning CLI (add/list/get/update/delete/quiz/stats). Invoke when user needs to manage vocabulary words, review vocabulary, or track language learning progress."
---

# i-rs-vocab

Vocabulary learning CLI tool for managing and reviewing vocabulary words with spaced repetition.

## Storage

- Config: `~/.config/i-rs/vocab.json`

## Commands

### add
Add a new vocabulary word.
```bash
i-rs-vocab add <WORD> <DEFINITION> [--tag] [--example] [--status]
```

### list
List vocabulary words.
```bash
i-rs-vocab list [--status STATUS] [--tag TAG]
```

### get
Show word details.
```bash
i-rs-vocab get <WORD>
```

### update
Update a word.
```bash
i-rs-vocab update <WORD> [--definition] [--status] [--tag] [--example] [--review]
```

### delete
Delete a word.
```bash
i-rs-vocab delete <WORD>
```

### quiz
Practice vocabulary.
```bash
i-rs-vocab quiz [--count N]
```

### stats
Show learning statistics.
```bash
i-rs-vocab stats
```

## Examples

```bash
# Add a word
i-rs-vocab add hello "greeting" --tag basics --example "Hello, how are you?"

# List words
i-rs-vocab list
i-rs-vocab list --status new
i-rs-vocab list --tag basics

# Quiz mode
i-rs-vocab quiz

# View stats
i-rs-vocab stats
```

## Status Values

- `new` - Newly added word (🆕)
- `learning` - Word being learned (📖)
- `mastered` - Word fully mastered (✅)
