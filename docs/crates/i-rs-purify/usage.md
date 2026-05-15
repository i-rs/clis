# i-rs-purify Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Add a filter replacement record.

```bash
i-rs-purify add <FILTER_TYPE> [OPTIONS]
```

Arguments:
- `FILTER_TYPE` - Type of filter (RO Membrane, Carbon Filter, Sediment Filter, Mineral Filter)

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List filter replacement records.

```bash
i-rs-purify list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-purify get <ID>
```

### delete

Delete a record.

```bash
i-rs-purify delete <ID>
```

## Filter Types

| Type | Description | Typical Lifespan |
|------|-------------|------------------|
| RO Membrane | Reverse osmosis membrane | 2-3 years |
| Carbon Filter | Activated carbon filter | 6-12 months |
| Sediment Filter | Pre-filter for sediment | 3-6 months |
| Mineral Filter | Post-filter adding minerals | 6-12 months |

### data

Manage data (export, import, clear).

```bash
i-rs-purify data export
i-rs-purify data import [FILE]
i-rs-purify data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
### example

Show usage examples.

```bash
i-rs-purify example
```
### skill

Show skill information.

```bash
i-rs-purify skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/purify.json`
- Linux: `~/.config/i-rs/purify.json`
- Windows: `~\AppData\Roaming\i-rs\purify.json`