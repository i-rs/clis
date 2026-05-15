# i-rs-bed Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Add a bed item replacement record.

```bash
i-rs-bed add <ITEM_TYPE> [OPTIONS]
```

Arguments:
- `ITEM_TYPE` - Type of item (mattress, pillow, duvet, mattress-protector)

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List bed item replacement records.

```bash
i-rs-bed list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-bed get <ID>
```

### delete

Delete a record.

```bash
i-rs-bed delete <ID>
```

## Bed Item Types

| Type | Description | Replacement Interval |
|------|-------------|---------------------|
| mattress | Mattress | 7-10 years |
| pillow | Pillow | 1-2 years |
| duvet | Duvet/comforter | 5-10 years |
| mattress-protector | Mattress protector | 2-3 years |

### data

Manage data (export, import, clear).

```bash
i-rs-bed data export
i-rs-bed data import [FILE]
i-rs-bed data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
### example

Show usage examples.

```bash
i-rs-bed example
```
### skill

Show skill information.

```bash
i-rs-bed skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/beds.json`
- Linux: `~/.config/i-rs/beds.json`
- Windows: `~\AppData\Roaming\i-rs\bed.json`