# i-rs-keys Examples

## Basic Usage

### Adding Keys

```bash
# Add GitHub token
i-rs-keys add "github-token" --type api-key --remark "GitHub personal access token"

# Add AWS credentials
i-rs-keys add "aws-access" --type aws-key --remark "Production AWS keys"

# Add SSH key reference
i-rs-keys add "ssh-work" --type ssh-key --remark "Work laptop SSH key"

# Add database password
i-rs-keys add "db-prod" --type password --remark "Production database"
```

### Listing Keys

```bash
# List all keys
i-rs-keys list

# Get specific key info
i-rs-keys get github-token
```

## Key Types

```bash
# API Keys
i-rs-keys add "stripe-api" --type api-key --tag payment --tag production

# AWS Keys
i-rs-keys add "aws-prod" --type aws-key --tag aws --tag production

# SSH Keys
i-rs-keys add "github-ssh" --type ssh-key --tag github

# Tokens
i-rs-keys add "openai-token" --type token --tag ai --tag api
```

## Managing Keys

```bash
# Update remarks
i-rs-keys update "github-token" --remark "Updated: new token"

# Delete old key
i-rs-keys delete "old-service"
```