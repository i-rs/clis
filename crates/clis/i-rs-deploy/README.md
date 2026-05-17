# i-rs-deploy

Deployment tracking CLI tool for tracking deployments, managing rollbacks, and viewing statistics.

## Features

- Cross-project and environment deployment tracking
- Deployment status logging (success/failed/rolling_back/rolled_back)
- Rollback management
- Statistics and deployment timeline
- Tag and remark support

## Install

```bash
npm install -g @i-rs/i-rs-deploy
# or
brew install i-rs/homebrew-tap/i-rs-deploy
```

## Quick Start

```bash
# Add a deployment record
i-rs-deploy add myapp production v1.2.3 --status success

# List all deployments
i-rs-deploy list

# View deployment details
i-rs-deploy get abc12345

# Rollback to previous version
i-rs-deploy rollback myapp production

# View statistics
i-rs-deploy stats
```

## Data Storage

- macOS: `~/.config/i-rs/deploy.json`
- Linux: `~/.config/i-rs/deploy.json`
- Windows: `~\AppData\Roaming\i-rs\deploy.json`

## License

MIT OR Apache-2.0
