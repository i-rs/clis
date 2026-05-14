# i-rs-step Usage

## Commands

### add

Record daily steps.

```bash
i-rs-step add <STEPS> [OPTIONS]
```

Arguments:
- `STEPS` - Number of steps

Options:
- `--distance <KM>` - Distance in kilometers
- `--date <DATE>` - Date (YYYY-MM-DD, default: today)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List step records.

```bash
i-rs-step list
```

Options:
- `--date <DATE>` - Filter by date

### get

Get record details.

```bash
i-rs-step get <DATE>
```

### delete

Delete a record.

```bash
i-rs-step delete <DATE>
```

### update

Update a record.

```bash
i-rs-step update <DATE> [OPTIONS]
```

Options:
- `--steps <STEPS>` - Update steps
- `--distance <KM>` - Update distance
- `-t, --tag <TAG>` - Add tags
- `-r, --remark <REMARK>` - Add remarks

## Data Storage

- macOS: `~/.config/i-rs/step.json`
- Linux: `~/.config/i-rs/step.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-step list
```