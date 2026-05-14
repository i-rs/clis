# i-rs-dose Usage

## Commands

### add

Record medicine intake.

```bash
i-rs-dose add <MEDICINE_NAME> --dosage <AMOUNT> --unit <UNIT> [OPTIONS]
```

Arguments:
- `MEDICINE_NAME` - Name of the medicine

Options:
- `--dosage <AMOUNT>` - Dosage amount
- `--unit <UNIT>` - Unit (tablet, ml, mg, IU, drop, capsule, etc.)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List medicine records.

```bash
i-rs-dose list
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get record details.

```bash
i-rs-dose get <ID>
```

### delete

Delete a record.

```bash
i-rs-dose delete <ID>
```

## Data Storage

- macOS: `~/.config/i-rs/dose.json`
- Linux: `~/.config/i-rs/dose.json`
- Windows: `~\AppData\Roaming\i-rs\dose.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-dose list
```