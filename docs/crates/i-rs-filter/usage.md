# i-rs-filter Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Add a filter cleaning record.

```bash
i-rs-filter add <APPLIANCE_NAME> <FILTER_TYPE> [OPTIONS]
```

Arguments:
- `APPLIANCE_NAME` - Name of the appliance
- `FILTER_TYPE` - Type of filter (HEPA, Carbon, Foam, Dust, etc.)

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List filter cleaning records.

```bash
i-rs-filter list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-filter get <ID>
```

### delete

Delete a record.

```bash
i-rs-filter delete <ID>
```

## Filter Types

| Type | Description |
|------|-------------|
| HEPA | High-efficiency particulate air filter |
| Carbon | Activated carbon filter |
| Foam | Foam filter |
| Dust | Dust collection filter |

### data

Manage data (export, import, clear).

```bash
i-rs-filter data export
i-rs-filter data import [FILE]
i-rs-filter data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
### example

Show usage examples.

```bash
i-rs-filter example
```
### skill

Show skill information.

```bash
i-rs-filter skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/filters.json`
- Linux: `~/.config/i-rs/filters.json`
- Windows: `~\AppData\Roaming\i-rs\filter.json`