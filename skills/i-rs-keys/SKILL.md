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
i-rs-keys add <NAME> <VALUE> <TYPE> [OPTIONS]
```

Arguments:
- `NAME` - Key name
- `VALUE` - Key value (stored securely in OS keychain)
- `TYPE` - Key type (api_key, aws_key, ssh_key, password, token, other)

Options:
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

## Examples

```bash
# Add API key (NAME VALUE TYPE --tag TAG --remark REMARK)
i-rs-keys add "github-token" "ghp_xxx" api_key --remark "GitHub PAT"
i-rs-keys add "openai-api" "sk_xxx" api_key --tag ai

# Add AWS credentials
i-rs-keys add "aws-access" "AKIAXXX" aws_key --tag production

# List keys
i-rs-keys list

# Get key (with value from keychain)
i-rs-keys get github-token --show-value
```