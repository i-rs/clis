# i-rs-keys Test Records

## Test Data

```bash
# API Keys
i-rs-keys add "github-token" --type api-key --remark "GitHub PAT"
i-rs-keys add "openai-api" --type api-key --tag ai

# AWS Keys
i-rs-keys add "aws-dev" --type aws-key --tag development
i-rs-keys add "aws-prod" --type aws-key --tag production

# SSH Keys
i-rs-keys add "github-ssh" --type ssh-key --tag github
i-rs-keys add "server-ssh" --type ssh-key --tag server

# Tokens
i-rs-keys add "slack-token" --type token --tag work
```