# i-rs-kv Examples

## Basic Usage

### Storing Values

```bash
# Store simple values
i-rs-kv add "username" --value "john"
i-rs-kv add "email" --value "john@example.com"
i-rs-kv add "theme" --value "dark"

# Store URLs
i-rs-kv add "api-url" --value "https://api.example.com"
i-rs-kv add "github" --value "https://github.com/john"
```

### Retrieving Values

```bash
# Get a value
i-rs-kv get username

# List all entries
i-rs-kv list
```

## Organization

### Using Tags

```bash
# Work-related
i-rs-kv add "work-email" --value "john@company.com" --tag work
i-rs-kv add "slack-url" --value "https://company.slack.com" --tag work

# Personal
i-rs-kv add "personal-email" --value "john@gmail.com" --tag personal
i-rs-kv add "netflix" --value "https://netflix.com" --tag personal

# List by tag
i-rs-kv list --tag work
```

## Configuration Storage

```bash
# Store app configurations
i-rs-kv add "editor" --value "vscode" --tag config --tag dev
i-rs-kv add "terminal" --value "iterm2" --tag config --tag dev
i-rs-kv add "shell" --value "zsh" --tag config --tag dev
```

## Managing Entries

```bash
# Update a value
i-rs-kv update "theme" --value "light"

# Delete entry
i-rs-kv delete "old-key"
```