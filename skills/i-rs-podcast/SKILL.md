---
name: "i-rs-podcast"
description: "Podcast and course tracking CLI (add/listen/update/stats). Invoke when user needs to track podcast episodes, manage course progress, or record learning notes."
---

# i-rs-podcast

Podcast and course tracking CLI tool for managing your audio/video learning content.

## Storage

- Config: `~/.config/i-rs/podcasts.json`

## Commands

### add
Add a new podcast or course.
```bash
i-rs-podcast add <NAME> [--author] [--duration] [--tag] [--remark] [--notes]
```

### list
List all podcasts with optional filters.
```bash
i-rs-podcast list [--status STATUS] [--tag TAG]
```

### get
Show detailed podcast information.
```bash
i-rs-podcast get <NAME>
```

### listen
Update listening progress with current position.
```bash
i-rs-podcast listen <NAME> --position <SECONDS> [--notes]
```

### update
Update podcast information.
```bash
i-rs-podcast update <NAME> [--author] [--duration] [--tag] [--remark] [--notes]
```

### delete
Delete a podcast from collection.
```bash
i-rs-podcast delete <NAME>
```

### stats
Show podcast collection statistics.
```bash
i-rs-podcast stats
```

## Status
- `not_started` (○): Not started
- `in_progress` (◐): Currently listening
- `completed` (●): Finished

## Examples

```bash
# Add podcast
i-rs-podcast add "The Daily" --author "NYT" --duration 3600

# Update progress
i-rs-podcast listen "The Daily" --position 1800

# List all
i-rs-podcast list

# View stats
i-rs-podcast stats
```
