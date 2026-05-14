# i-rs-petbath Usage

## Commands

### add

Add a pet bath record.

```bash
i-rs-petbath add <PET_NAME> [OPTIONS]
```

Arguments:
- `PET_NAME` - Name of the pet

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List pet bath records.

```bash
i-rs-petbath list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-petbath get <ID>
```

### delete

Delete a record.

```bash
i-rs-petbath delete <ID>
```

## Data Storage

- macOS: `~/.config/i-rs/petbath.json`
- Linux: `~/.config/i-rs/petbath.json`
- Windows: `~\AppData\Roaming\i-rs\petbath.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-petbath list
```