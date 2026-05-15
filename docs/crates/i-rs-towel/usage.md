# i-rs-towel Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Add a towel replacement record.

```bash
i-rs-towel add <TOWEL_TYPE> [OPTIONS]
```

Arguments:
- `TOWEL_TYPE` - Type of towel (bath, face, hand, beach, sports)

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List towel replacement records.

```bash
i-rs-towel list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-towel get <ID>
```

### delete

Delete a record.

```bash
i-rs-towel delete <ID>
```

## Towel Types

| Type | Description | Replacement Interval |
|------|-------------|---------------------|
| bath | Bath towel | 2-5 years |
| face | Face towel | 1-2 years |
| hand | Hand towel | 1-2 years |
| beach | Beach towel | 3-5 years |
| sports | Sports towel | 1-2 years |

### data

Manage data (export, import, clear).

```bash
i-rs-towel data export
i-rs-towel data import [FILE]
i-rs-towel data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
### example

Show usage examples.

```bash
i-rs-towel example
```
### skill

Show skill information.

```bash
i-rs-towel skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/towels.json`
- Linux: `~/.config/i-rs/towels.json`
- Windows: `~\AppData\Roaming\i-rs\towel.json`