# i-rs-toothbrush Usage

## Global Flags

- `--json` — Output in JSON format

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

### data

Manage data (export, import, clear).

```bash
i-rs-toothbrush data export
i-rs-toothbrush data import [FILE]
i-rs-toothbrush data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data

### example

Show usage examples.

```bash
i-rs-toothbrush example
```

### skill

Show skill information.

```bash
i-rs-toothbrush skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/toothbrushes.json`
- Linux: `~/.config/i-rs/toothbrushes.json`
- Windows: `~\AppData\Roaming\i-rs\toothbrush.json`