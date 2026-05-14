# i-rs-vocab Usage

## Commands

### add

Add a new vocabulary word.

```bash
i-rs-vocab add <WORD> <DEFINITION> [OPTIONS]

Options:
  -e, --example <TEXT>    Example sentence (repeatable)
  -s, --status <STATUS>  Initial status (new/learning/mastered)
  -t, --tag <TAG>        Tags (repeatable)
  -r, --remark <TEXT>    Remarks (repeatable)
```

Examples:

```bash
i-rs-vocab add hello "greeting; hello world" --tag basic
i-rs-vocab add ephemeral "lasting for a very short time" --tag advanced
i-rs-vocab add serendipity "the occurrence of events by chance in a happy way" --tag vocabulary
```

### list

List vocabulary words.

```bash
i-rs-vocab list [OPTIONS]

Options:
  -s, --status <STATUS>  Filter by status (new/learning/mastered)
  -t, --tag <TAG>        Filter by tag
```

Examples:

```bash
i-rs-vocab list
i-rs-vocab list --status new
i-rs-vocab list --status learning
i-rs-vocab list --tag basic
```

### get

Show word details.

```bash
i-rs-vocab get <WORD>
```

### update

Update a word.

```bash
i-rs-vocab update <WORD> [OPTIONS]

Options:
  -d, --definition <TEXT>  Update definition
  -e, --example <TEXT>    Update examples (repeatable)
  -s, --status <STATUS>   Update status
  -t, --tag <TAG>         Update tags (repeatable)
  -r, --remark <TEXT>     Update remarks (repeatable)
  --review                Increment review count
```

### delete

Delete a word.

```bash
i-rs-vocab delete <WORD>
```

### quiz

Practice vocabulary with quiz mode.

```bash
i-rs-vocab quiz [OPTIONS]

Options:
  -c, --count <N>  Number of words to quiz (default: 5)
```

### stats

Show learning statistics.

```bash
i-rs-vocab stats
```

## Status Values

| Status | Description |
|--------|-------------|
| `new` | Newly added word |
| `learning` | Word being actively learned |
| `mastered` | Word fully mastered |

## Global Options

| Option | Description |
|--------|-------------|
| `--json` | Output in JSON format |
| `-h, --help` | Show help |
| `-V, --version` | Show version |

## Data Storage

- macOS: `~/.config/i-rs/vocab.json`
- Linux: `~/.config/i-rs/vocab.json`
- Windows: `~\AppData\Roaming\i-rs\vocab.json`
