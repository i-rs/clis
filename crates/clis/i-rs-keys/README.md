# i-rs-keys

API keys and secrets management CLI tool using OS keychain for secure storage.

## Features

- Secure key storage using OS keychain
- Support for multiple key types
- Tag support for categorization
- Quick access to stored keys

## Install

```bash
npm install -g @i-rs/i-rs-keys
# or
brew install i-rs/homebrew-tap/i-rs-keys
```

## Quick Start

```bash
# Add a key (NAME VALUE TYPE --tag TAG --remark REMARK)
i-rs-keys add "github-token" "ghp_xxx" api_key --tag coding --remark "GitHub PAT"
i-rs-keys add "aws-access" "AKIAXXX" aws_key --tag cloud --remark "AWS credentials"

# List all keys
i-rs-keys list

# Get key details
i-rs-keys get github-token
```

## Security

Keys are stored securely in the OS keychain (macOS Keychain, Linux Secret Service, or Windows Credential Manager).

## Data Storage

- Keys stored in OS keychain
- Metadata in: `~/.config/i-rs/keys.json`

## License

MIT OR Apache-2.0