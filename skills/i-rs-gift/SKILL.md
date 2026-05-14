---
name: "i-rs-gift"
description: "Manages gift records (add/list/get/delete/stats). Invoke when user needs to track gifts given or received, or view gift exchange statistics."
---

# i-rs-gift

Gift management CLI tool for tracking gifts given and received.

## Storage

- Config: `~/.config/i-rs/gifts.json`

## Commands

### add

Add a new gift record.

```bash
i-rs-gift add <NAME> <TYPE> <RECIPIENT> <OCCASION> <VALUE> <DATE> [--tag TAG] [--remark REMARK]
```

TYPE: `sent` or `received`

### list

List all gifts or filter by type/tag.

```bash
i-rs-gift list [--type TYPE] [--tag TAG]
```

### get

Get gift details.

```bash
i-rs-gift get <NAME>
```

### delete

Delete a gift.

```bash
i-rs-gift delete <NAME>
```

### stats

Show gift statistics.

```bash
i-rs-gift stats
```

Shows:
- Total gifts sent/received
- Total value sent/received
- Average values
- Balance (received - sent)
- Top occasions
- Top recipients

### example

Show usage examples.

```bash
i-rs-gift example
```

## Examples

```bash
# Add a sent gift
i-rs-gift add "Birthday Watch" sent "Mom" birthday 500 2024-12-25 --tag family

# Add a received gift
i-rs-gift add "AirPods Pro" received "Boss" christmas 1200 2024-12-25 --tag work

# List all gifts
i-rs-gift list

# List sent gifts only
i-rs-gift list --type sent

# View statistics
i-rs-gift stats
```
