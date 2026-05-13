# i-rs-password Examples

This page provides comprehensive examples for all i-rs-password commands.

## Basic Password Management

### Adding Password Entries

```bash
# Add a simple website account
i-rs-password add github https://github.com --account user@example.com --password ghp_xxxxx --tag work --remark "GitHub account"

# Add a database credential
i-rs-password add prod-mysql mysql://db.example.com:3306 --account admin --password secure123 --tag production --tag database --remark "Production MySQL"

# Add a PostgreSQL database
i-rs-password add prod-postgres postgresql://pg.example.com:5432 --account pgadmin --password pgpass123 --tag production --tag database

# Add an API key
i-rs-password add stripe-api https://api.stripe.com --account api --password sk_live_xxxx --tag payment --tag important

# Add AWS credentials
i-rs-password add aws-console https://aws.amazon.com --account admin@example.com --password aws_pass_123 --tag cloud --tag aws

# Add a server SSH credential (non-standard port)
i-rs-password add prod-server ssh://192.168.1.100:2222 --user admin --password server_pass --tag production --tag server
```

### Listing Passwords

```bash
# List all passwords
i-rs-password list

# Filter by work tag
i-rs-password list --tag work

# Filter by production tag
i-rs-password list --tag production

# Filter by database tag
i-rs-password list --tag database
```

### Retrieving Passwords

```bash
# Get password details (hidden by default)
i-rs-password get github

# Show password in output
i-rs-password get github --show-password

# Get database credentials
i-rs-password get prod-mysql --show-password
```

### Updating Passwords

```bash
# Update just the password
i-rs-password update github --password new_ghp_token

# Update account username
i-rs-password update github --account newemail@example.com

# Update URL
i-rs-password update github --url https://github.com/enterprise

# Replace all tags
i-rs-password update github --tag personal --tag github

# Multiple field update
i-rs-password update github --account updated@example.com --password newpass --tag important
```

### Deleting Passwords

```bash
# Delete a single entry
i-rs-password delete github

# Delete multiple
i-rs-password delete old-api-key
i-rs-password delete test-account
```

## Organized Password Management

### By Category

```bash
# Development passwords
i-rs-password add local-mysql mysql://localhost:3306 --account root --password root --tag dev --tag mysql
i-rs-password add local-redis redis://localhost:6379 --account "" --password redis_pass --tag dev --tag redis

# Cloud services
i-rs-password add aws-dev https://console.aws.amazon.com --account dev@example.com --password aws_dev_pass --tag dev --tag aws
i-rs-password add aws-prod https://console.aws.amazon.com --account prod@example.com --password aws_prod_pass --tag production --tag aws

# Payment gateways
i-rs-password add stripe-test https://dashboard.stripe.com --account test --password sk_test_xxx --tag test --tag payment
i-rs-password add stripe-prod https://dashboard.stripe.com --account prod --password sk_live_xxx --tag production --tag payment
```

### By Project

```bash
# Project Alpha
i-rs-password add alpha-db mysql://alpha-db.example.com --account alpha --password alpha_pass --tag project-alpha --tag database
i-rs-password add alpha-api https://api.alpha.example.com --account service --password alpha_api_key --tag project-alpha --tag api

# Project Beta
i-rs-password add beta-db postgresql://beta-db.example.com --account beta --password beta_pass --tag project-beta --tag database
i-rs-password add beta-s3 https://s3.amazonaws.com/bucket --account beta --password beta_s3_key --tag project-beta --tag storage
```

## Real-World Scenarios

### Daily Development Workflow

```bash
# Morning: List all dev credentials
i-rs-password list --tag dev

# Get database password for project
i-rs-password get alpha-db --show-password

# Update after rotation
i-rs-password update alpha-db --password new_rotated_password
```

### Onboarding New Developer

```bash
# Add all necessary credentials for new developer
i-rs-password add github-org https://github.com/org --account newdev@company.com --password temp_pass --tag onboarding --tag github
i-rs-password add jira https://company.atlassian.net --account newdev@company.com --password temp_pass --tag onboarding --tag jira
i-rs-password add slack-api https://api.slack.com --account bot --password xoxb-xxx --tag onboarding --tag slack
```

### Password Rotation

```bash
# Rotate all production database passwords
i-rs-password update prod-mysql --password $(openssl rand -base64 20)
i-rs-password update prod-postgres --password $(openssl rand -base64 20)
i-rs-password update prod-mongodb --password $(openssl rand -base64 20)

# Verify rotation
i-rs-password list --tag production
```

### Emergency Access

```bash
# When someone needs emergency access to production
i-rs-password get prod-database --show-password

# Add emergency contact credential
i-rs-password add emergency-oncall https://company.pagerduty.com --account oncall@company.com --password emergency_pass --tag emergency --tag oncall
```

## Advanced Usage

### Bulk Operations via Script

```bash
#!/bin/bash
# backup-passwords.sh - Export (never with actual passwords)

echo "=== Password Categories ==="
echo "Development:"
i-rs-password list --tag dev
echo ""
echo "Production:"
i-rs-password list --tag production
echo ""
echo "Cloud:"
i-rs-password list --tag cloud
```

### Using with Password Managers

```bash
# Generate strong password and store
NEW_PASS=$(openssl rand -base64 24)
i-rs-password update github --password "$NEW_PASS"

# Store the new password in a file (for backup purposes)
echo "$NEW_PASS" | gpg -c > github-password.gpg
```

### API Integration

```bash
# Get password for API call (using process substitution)
curl -u $(i-rs-password get github --show-password | grep password | awk '{print $2}') https://api.github.com/user

# Actually better approach - use in scripts carefully
PASS=$(i-rs-password get api-key --show-password 2>/dev/null | grep "Password:" | cut -d: -f2 | tr -d ' ')
curl -u "api_user:$PASS" https://api.example.com
```

## Security Best Practices

### Using Keychain Features

```bash
# Password is automatically stored in OS keychain
# On macOS, can view in Keychain Access
# On Linux, can view in Seahorse (gnome-keyring)
# On Windows, can view in Credential Manager

# Verify password is in keychain (macOS)
security find-internet-password -s github.com
```

### Environment-Specific Separation

```bash
# Development environment - lower security acceptable
i-rs-password add dev-api https://dev-api.example.com --account dev --password dev_pass --tag dev

# Staging environment - medium security
i-rs-password add staging-api https://staging-api.example.com --account staging --password staging_pass --tag staging

# Production environment - maximum security
i-rs-password add prod-api https://api.example.com --account prod --password $(openssl rand -base64 32) --tag production --tag critical
```

### Audit Trail

```bash
# Add remarks for audit
i-rs-password update github --remark "Password rotated on 2024-01-15 by admin"
i-rs-password update github --remark "Requested by John for project X"

# Check all entries with remarks
i-rs-password list
```

## Troubleshooting

### Password Not in Keychain

```bash
# Check if password is stored
# macOS
security find-internet-password -s github.com

# Linux (with dbus)
secret-tool search service github.com

# If missing, re-add password
i-rs-password update github --password actual_password
```

### Entry Not Found

```bash
# Check all entries
i-rs-password list

# Try different tags
i-rs-password list --tag work
i-rs-password list --tag personal
```

### Keychain Access Denied

```bash
# macOS: Unlock keychain
security -v unlock-keychain ~/Library/Keychains/login.keychain

# Linux: Check if secret service is running
dbus-send --session --dest=org.freedesktop.secrets --type=method_call --print-reply /org/freedesktop/secrets org.freedesktop.Secrets.Service.OpenSession