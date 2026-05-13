# i-rs-note

Note management CLI tool for managing notes locally.

## Overview

i-rs-note helps you manage simple text notes with title and tag support. Perfect for quick notes, todo lists, ideas, and reference information.

## Quick Start

```bash
# Add a note
i-rs-note add todo --title "My Tasks" --tag work --content "Buy groceries" --content "Call mom"

# Add a note with multiple tags
i-rs-note add ideas --title "Project Ideas" --tag project --tag brainstorm --content "Idea 1" --content "Idea 2"

# List notes
i-rs-note list

# List by tag
i-rs-note list --tag work

# Get note details
i-rs-note get todo

# Update note
i-rs-note update todo --content "New task"

# Delete note
i-rs-note delete todo
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-note

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-note
```

## Data Storage

- macOS: `~/.config/i-rs/notes.json`
- Linux: `~/.config/i-rs/notes.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records