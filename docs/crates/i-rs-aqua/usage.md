# i-rs-aqua Usage

## Commands

### add

Add a water change record.

```bash
i-rs-aqua add [OPTIONS]
```

Options:
- `-t, --tank-size <SIZE>` - Tank size in liters
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List water change records.

```bash
i-rs-aqua list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-aqua get <ID>
```

### delete

Delete a record.

```bash
i-rs-aqua delete <ID>
```

## Data Storage

- macOS: `~/.config/i-rs/aqua.json`
- Linux: `~/.config/i-rs/aqua.json`
- Windows: `~\AppData\Roaming\i-rs\aqua.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-aqua list
```