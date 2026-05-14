# i-rs-pig Usage

## Commands

### add

Record a craving or indulgence.

```bash
i-rs-pig add <FOOD_NAME> [OPTIONS]
```

Arguments:
- `FOOD_NAME` - Name of the food

Options:
- `--description <DESC>` - Description
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List craving records.

```bash
i-rs-pig list
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-pig get <ID>
```

### delete

Delete a record.

```bash
i-rs-pig delete <ID>
```

## Data Storage

- macOS: `~/.config/i-rs/pig.json`
- Linux: `~/.config/i-rs/pig.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-pig list
```