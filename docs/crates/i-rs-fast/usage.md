# i-rs-fast Usage

## Commands

### add

Start a fasting session.

```bash
i-rs-fast add <TARGET_HOURS> [OPTIONS]
```

Arguments:
- `TARGET_HOURS` - Target fasting duration in hours

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List fasting records.

```bash
i-rs-fast list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-fast get <ID>
```

### delete

Delete a record.

```bash
i-rs-fast delete <ID>
```

## Fasting Protocols

| Protocol | Fasting Hours | Eating Window |
|----------|--------------|---------------|
| 16:8 | 16 hours | 8 hours |
| 18:6 | 18 hours | 6 hours |
| 20:4 | 20 hours | 4 hours |
| OMAD | 23 hours | 1 hour |

## Data Storage

- macOS: `~/.config/i-rs/fast.json`
- Linux: `~/.config/i-rs/fast.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-fast list
```