# i-rs-bookmark

Bookmark management CLI tool for managing URLs and credentials locally.

## Install

```bash
npm install -g @i-rs/i-rs-bookmark
# or
brew install i-rs/homebrew-tap/i-rs-bookmark
```

## Security

**Passwords are stored securely in the OS keychain, never in the JSON config file.**

Supported keychain backends:
- macOS: Keychain
- Linux: Secret Service / keyutils
- Windows: Credential Manager

## Usage

### Add Bookmark

```bash
i-rs-bookmark add <NAME> <URL> [OPTIONS]
```

Options:
- `-u, --account <ACCOUNT>` - Account/username (optional)
- `-p, --password <PASSWORD>` - Password (stored securely in keychain, optional)
- `-t, --tag <TAG>` - Tags (can be specified multiple times)
- `-r, --remark <REMARK>` - Remarks (can be specified multiple times)

### List Bookmarks

```bash
i-rs-bookmark list [OPTIONS]
```

Options:
- `-t, --tag <TAG>` - Filter by tag

### Get Bookmark Details

```bash
i-rs-bookmark get <NAME> [OPTIONS]
```

Options:
- `-s, --show-password` - Show password from keychain

### Update Bookmark

```bash
i-rs-bookmark update <NAME> [OPTIONS]
```

Options:
- `--url <URL>` - New URL
- `-u, --account <ACCOUNT>` - New account
- `-p, --password <PASSWORD>` - New password (stored securely in keychain)
- `-t, --tag <TAG>` - New tags
- `-r, --remark <REMARK>` - New remarks

### Delete Bookmark

```bash
i-rs-bookmark delete <NAME>
```

## Examples

```bash
# Add a simple bookmark (no account/password)
i-rs-bookmark add github https://github.com --tag work --remark "GitHub"

# Add a bookmark with credentials
i-rs-bookmark add aws https://aws.amazon.com --account admin@example.com --password secret123 --tag cloud --remark "AWS account"

# List all bookmarks
i-rs-bookmark list

# List bookmarks by tag
i-rs-bookmark list --tag work

# Get bookmark details (password hidden)
i-rs-bookmark get github

# Get bookmark details with password visible
i-rs-bookmark get aws --show-password

# Update bookmark
i-rs-bookmark update github --remark "Updated GitHub link"

# Delete a bookmark
i-rs-bookmark delete github
```

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/bookmarks.json`
- Linux: `~/.config/i-rs/bookmarks.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

**Passwords are NEVER stored in the config file. They go to the OS keychain.**

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-bookmark list
```

## License

MIT OR Apache-2.0
