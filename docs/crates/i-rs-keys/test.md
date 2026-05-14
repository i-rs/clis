# i-rs-keys Test Records

## Test Data

```bash
# API Keys
i-rs-keys add "github-token" "ghp_xxx" api_key --tag github
i-rs-keys add "openai-api" "sk_xxx" api_key --tag ai

# AWS Keys
i-rs-keys add "aws-dev" "AKIA_DEV_XXX" aws_key --tag development
i-rs-keys add "aws-prod" "AKIA_PROD_XXX" aws_key --tag production

# SSH Keys
i-rs-keys add "github-ssh" "~/.ssh/github_key" ssh_key --tag github
i-rs-keys add "server-ssh" "~/.ssh/server_key" ssh_key --tag server

# Tokens
i-rs-keys add "slack-token" "xoxb-xxx" token --tag work

# List all
i-rs-keys list

# Get with value
i-rs-keys get github-token --show-value
```