# i-rs-water Usage

## Commands

### add

Add a water intake record.

```bash
i-rs-water add <AMOUNT_ML> [OPTIONS]
```

Arguments:
- `AMOUNT_ML` - Water amount in milliliters

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List water intake records.

```bash
i-rs-water list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-water get <ID>
```

### delete

Delete a record.

```bash
i-rs-water delete <ID>
```

## Data Storage

- macOS: `~/.config/i-rs/water.json`
- Linux: `~/.config/i-rs/water.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-water list
```