# i-rs-walkdog Usage

## Commands

### add

Add a dog walk record.

```bash
i-rs-walkdog add <DOG_NAME> <DURATION_MINUTES> [OPTIONS]
```

Arguments:
- `DOG_NAME` - Name of the dog
- `DURATION_MINUTES` - Walk duration in minutes

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List dog walk records.

```bash
i-rs-walkdog list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-walkdog get <ID>
```

### delete

Delete a record.

```bash
i-rs-walkdog delete <ID>
```

## Data Storage

- macOS: `~/.config/i-rs/walkdog.json`
- Linux: `~/.config/i-rs/walkdog.json`
- Windows: `~\AppData\Roaming\i-rs\walkdog.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-walkdog list
```