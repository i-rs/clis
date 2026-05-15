# i-rs-vocab Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Add a new vocabulary word.

```bash
i-rs-vocab add <WORD> <DEFINITION> [OPTIONS]
```

**Arguments:**
- `WORD` - The vocabulary word
- `DEFINITION` - Word definition

**Options:**
- `-e, --example <TEXT>` - Example sentences (repeatable)
- `-s, --status <STATUS>` - Learning status (learning, reviewing, mastered)
- `-t, --tag <TAG>` - Tags (repeatable)
- `-r, --remark <TEXT>` - Remarks (repeatable)

### list

List vocabulary words.

```bash
i-rs-vocab list [OPTIONS]
```

**Options:**
- `-s, --status <STATUS>` - Filter by status
- `-t, --tag <TAG>` - Filter by tag

### get

Get word details.

```bash
i-rs-vocab get <WORD>
```

### update

Update a word.

```bash
i-rs-vocab update <WORD> [OPTIONS]
```

### delete

Delete a word.

```bash
i-rs-vocab delete <WORD>
```

### quiz

Start a vocabulary quiz.

```bash
i-rs-vocab quiz [OPTIONS]
```

**Options:**
- `-c, --count <N>` - Number of questions

### stats

Show vocabulary statistics.

```bash
i-rs-vocab stats
```

### data

Manage data (export, import, clear).

```bash
i-rs-vocab data export
i-rs-vocab data import [FILE]
i-rs-vocab data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data

### example

Show usage examples.

```bash
i-rs-vocab example
```

### skill

Show skill information.

```bash
i-rs-vocab skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/vocab.json`
- Linux: `~/.config/i-rs/vocab.json`
- Windows: `~\AppData\Roaming\i-rs\vocab.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-vocab list
```
