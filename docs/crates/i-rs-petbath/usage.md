# i-rs-petbath Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Add a pet bath record.

```bash
i-rs-petbath add <PET_NAME> [OPTIONS]
```

Arguments:
- `PET_NAME` - Name of the pet

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List pet bath records.

```bash
i-rs-petbath list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-petbath get <ID>
```

### delete

Delete a record.

```bash
i-rs-petbath delete <ID>
```

### data

Manage data (export, import, clear).

```bash
i-rs-petbath data export
i-rs-petbath data import [FILE]
i-rs-petbath data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
### example

Show usage examples.

```bash
i-rs-petbath example
```
### skill

Show skill information.

```bash
i-rs-petbath skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/petbath.json`
- Linux: `~/.config/i-rs/petbath.json`
- Windows: `~\AppData\Roaming\i-rs\petbath.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-petbath list
```