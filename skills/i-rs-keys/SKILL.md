---
name: "i-rs-keys"
description: "Manages API keys and secrets using OS keychain. Invoke when user wants to store or retrieve API keys securely."
---

# i-rs-keys

API keys and secrets management CLI tool.

## Storage

- Keys: OS Keychain (secure)
- Metadata: `~/.config/i-rs/keys.json`

## Commands

### add

Add a key entry.

```bash
i-rs-keys add <NAME> --type <TYPE> [OPTIONS]
```

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

## Examples

```bash
# Add API key
i-rs-keys add "github-token" --type api-key --remark "GitHub PAT"

# Add AWS credentials
i-rs-keys add "aws-access" --type aws-key --remark "Production keys"

# List keys
i-rs-keys list

# Get key
i-rs-keys get github-token
```