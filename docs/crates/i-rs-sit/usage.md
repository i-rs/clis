# i-rs-sit Usage

## Commands

### add

Add a sitting duration record.

```bash
i-rs-sit add <DURATION_MINUTES> [OPTIONS]
```

Arguments:
- `DURATION_MINUTES` - Duration in minutes

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List sitting records.

```bash
i-rs-sit list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-sit get <ID>
```

### delete

Delete a record.

```bash
i-rs-sit delete <ID>
```

## Data Storage

- macOS: `~/.config/i-rs/sit.json`
- Linux: `~/.config/i-rs/sit.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-sit list
```