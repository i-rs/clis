# i-rs-tick Usage

## Commands

### add

Record task duration.

```bash
i-rs-tick add <TASK_NAME> --duration <SECONDS> [OPTIONS]
```

Arguments:
- `TASK_NAME` - Name of the task

Options:
- `--duration <SECONDS>` - Duration in seconds
- `--description <DESC>` - Description
- `--started-at <DATETIME>` - Start time (YYYY-MM-DD HH:MM:SS)
- `--ended-at <DATETIME>` - End time (YYYY-MM-DD HH:MM:SS)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List duration records.

```bash
i-rs-tick list
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-tick get <ID>
```

### delete

Delete a record.

```bash
i-rs-tick delete <ID>
```

## Duration Format

Durations are specified in seconds:
- 60 seconds = 1 minute
- 3600 seconds = 1 hour
- 7200 seconds = 2 hours

## Data Storage

- macOS: `~/.config/i-rs/tick.json`
- Linux: `~/.config/i-rs/tick.json`
- Windows: `~\AppData\Roaming\i-rs\tick.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-tick list
```