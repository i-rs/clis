# i-rs-keys Usage

## Commands

### add

Add a key entry.

```bash
i-rs-keys add <NAME> <VALUE> <TYPE> [OPTIONS]
```

Arguments:
- `NAME` - Key name
- `VALUE` - Key value (will be stored in OS keychain)
- `TYPE` - Key type (api_key, aws_key, ssh_key, password, token, other)

Options:
- `-t, --tag <TAG>` - Tags (can be repeated)
- `-r, --remark <REMARK>` - Remarks (can be repeated)

### list

List all key entries.

```bash
i-rs-keys list
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### get

Get key entry details.

```bash
i-rs-keys get <NAME>
```

Options:
- `-s, --show-value` - Show the key value from keychain

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
- `--key-value <VALUE>` - Update key value
- `--type <TYPE>` - Update key type
- `-t, --tag <TAG>` - Add tags
- `-r, --remark <REMARK>` - Add remarks

## Key Types

- `api_key` - API key
- `aws_key` - AWS credentials
- `ssh_key` - SSH key
- `password` - Password
- `token` - Token
- `other` - Other type

## Data Storage

- Keys stored in OS Keychain (secure)
- Metadata: `~/.config/i-rs/keys.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-keys list
```