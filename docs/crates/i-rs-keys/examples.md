# i-rs-keys Examples

## Basic Usage

### Adding Keys

```bash
# Add GitHub token (NAME VALUE TYPE --tag TAG --remark REMARK)
i-rs-keys add "github-token" "ghp_xxx" api_key --remark "GitHub personal access token"

# Add AWS credentials
i-rs-keys add "aws-access" "AKIAXXX" aws_key --remark "Production AWS keys"

# Add SSH key reference
i-rs-keys add "ssh-work" "~/.ssh/id_rsa" ssh_key --remark "Work laptop SSH key"

# Add database password
i-rs-keys add "db-prod" "secret_password" password --remark "Production database"
```

### Listing Keys

```bash
# List all keys
i-rs-keys list

# Filter by tag
i-rs-keys list --tag production

# Get specific key info
i-rs-keys get github-token

# Show key value from keychain
i-rs-keys get github-token --show-value
```

## Key Types

```bash
# API Keys
i-rs-keys add "stripe-api" "sk_live_xxx" api_key --tag payment --tag production

# AWS Keys
i-rs-keys add "aws-prod" "AKIAXXX" aws_key --tag aws --tag production

# SSH Keys
i-rs-keys add "github-ssh" "~/.ssh/github_key" ssh_key --tag github

# Tokens
i-rs-keys add "openai-token" "sk-xxx" token --tag ai --tag api
```

## Managing Keys

```bash
# Update key value
i-rs-keys update "github-token" --key-value "ghp_new_xxx"

# Update key type
i-rs-keys update "github-token" --type token

# Add tags
i-rs-keys update "github-token" --tag important

# Delete old key
i-rs-keys delete "old-service"
```

## Security Notes

- All key values are stored in OS keychain (macOS Keychain, Linux Secret Service, or Windows Credential Manager)
- Only metadata (name, type, tags, remarks) is stored in the JSON file
- Use `--show-value` to retrieve values from keychain when needed