# i-rs-toothbrush Usage

## Commands

### add

Add a toothbrush replacement record.

```bash
i-rs-toothbrush add <BRUSH_TYPE> [OPTIONS]
```

Arguments:
- `BRUSH_TYPE` - Type of toothbrush (Electric, Manual, Kids, Interdental)

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List toothbrush replacement records.

```bash
i-rs-toothbrush list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-toothbrush get <ID>
```

### delete

Delete a record.

```bash
i-rs-toothbrush delete <ID>
```

## Brush Types

| Type | Description | Replacement Interval |
|------|-------------|---------------------|
| Electric | Electric toothbrush head | Every 3 months |
| Manual | Regular toothbrush | Every 3 months |
| Kids | Children's toothbrush | Every 3-4 months |
| Interdental | Interdental brush | Every 2-4 weeks |

## Data Storage

- macOS: `~/.config/i-rs/toothbrushes.json`
- Linux: `~/.config/i-rs/toothbrushes.json`
- Windows: `~\AppData\Roaming\i-rs\toothbrush.json`