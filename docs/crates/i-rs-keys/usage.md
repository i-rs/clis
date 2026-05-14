# i-rs-keys Usage

## Commands

### add

Add a key entry.

```bash
i-rs-keys add <NAME> --type <TYPE> [OPTIONS]
```

Arguments:
- `NAME` - Key name

Options:
- `--type <TYPE>` - Key type (api-key, aws-key, ssh-key, password, token, other)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List all key entries.

```bash
i-rs-keys list
```

### get

Get key entry details.

```bash
i-rs-keys get <NAME>
```

### delete

Delete a key entry.

```bash
i-rs-keys delete <NAME>
```

### update

Update a key entry.

```bash
i-rs-keys update <NAME> [OPTIONS]
```

Options:
- `--type <TYPE>` - Update key type
- `-t, --tag <TAG>` - Add tags
- `-r, --remark <REMARK>` - Add remarks

## Data Storage

- Keys stored in OS Keychain
- Metadata: `~/.config/i-rs/keys.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-keys list
```