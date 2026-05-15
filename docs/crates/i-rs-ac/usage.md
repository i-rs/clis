# i-rs-ac Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Add an AC cleaning record.

```bash
i-rs-ac add <LOCATION> [OPTIONS]
```

Arguments:
- `LOCATION` - Location of the AC unit (e.g., Living Room, Bedroom)

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List AC cleaning records.

```bash
i-rs-ac list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-ac get <ID>
```

### delete

Delete a record.

```bash
i-rs-ac delete <ID>
```

### data

Manage data (export, import, clear).

```bash
i-rs-ac data export
i-rs-ac data import [FILE]
i-rs-ac data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
### example

Show usage examples.

```bash
i-rs-ac example
```
### skill

Show skill information.

```bash
i-rs-ac skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/ac.json`
- Linux: `~/.config/i-rs/ac.json`
- Windows: `~\AppData\Roaming\i-rs\ac.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-ac list
```