# i-rs-ac Usage

## Commands

### add

Add an AC cleaning record.

```bash
i-rs-ac add <LOCATION> [OPTIONS]
```

Arguments:
- `LOCATION` - Location of the AC unit (e.g., Living Room, Bedroom)

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List AC cleaning records.

```bash
i-rs-ac list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-ac get <ID>
```

### delete

Delete a record.

```bash
i-rs-ac delete <ID>
```

## Data Storage

- macOS: `~/.config/i-rs/ac.json`
- Linux: `~/.config/i-rs/ac.json`
- Windows: `~\AppData\Roaming\i-rs\ac.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-ac list
```