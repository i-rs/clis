# i-rs-kv Examples

## Basic Usage

### Storing Values

```bash
# Store simple values
i-rs-kv add username john
i-rs-kv add email john@example.com
i-rs-kv add theme dark

# Store URLs
i-rs-kv add api-url "https://api.example.com"
i-rs-kv add github "https://github.com/john"
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
i-rs-kv add work-email john@company.com --tag work
i-rs-kv add slack-url "https://company.slack.com" --tag work

# Personal
i-rs-kv add personal-email john@gmail.com --tag personal
i-rs-kv add netflix "https://netflix.com" --tag personal

# List by tag
i-rs-kv list --tag work
```

## Configuration Storage

```bash
# Store app configurations
i-rs-kv add editor vscode --tag config --tag dev
i-rs-kv add terminal iterm2 --tag config --tag dev
i-rs-kv add shell zsh --tag config --tag dev
```

## Managing Entries

```bash
# Update a value
i-rs-kv update theme --value light

# Delete entry
i-rs-kv delete old-key
```

## JSON Output

```bash
# List as JSON
i-rs-kv list --json

# Get single entry as JSON
i-rs-kv get username --json
```

## Data Management

```bash
# Export all data
i-rs-kv data export

# Import from file
i-rs-kv data import backup.json

# Import from stdin
cat backup.json | i-rs-kv data import

# Clear all data
i-rs-kv data clear
```
