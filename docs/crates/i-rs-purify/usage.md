# i-rs-purify Usage

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

## Data Storage

- macOS: `~/.config/i-rs/purify.json`
- Linux: `~/.config/i-rs/purify.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`