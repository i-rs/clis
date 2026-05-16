# i-rs-pig Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Record a craving or indulgence.

```bash
i-rs-pig add <FOOD_NAME> [OPTIONS]
```

Arguments:
- `FOOD_NAME` - Name of the food

Options:
- `--description <DESC>` - Description
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List craving records.

```bash
i-rs-pig list
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-pig get <ID>
```

### delete

Delete a record.

```bash
i-rs-pig delete <ID>
```

### update

Update a craving record.

```bash
i-rs-pig update <ID> [OPTIONS]
```

Options:
- `--food-name <NAME>` - Food name
- `--description <DESC>` - Description
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### data

Manage data (export, import, clear).

```bash
i-rs-pig data export
i-rs-pig data import [FILE]
i-rs-pig data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
### example

Show usage examples.

```bash
i-rs-pig example
```
### skill

Show skill information.

```bash
i-rs-pig skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/pig.json`
- Linux: `~/.config/i-rs/pig.json`
- Windows: `~\AppData\Roaming\i-rs\pig.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-pig list
```