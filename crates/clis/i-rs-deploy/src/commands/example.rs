pub fn handle_example() {
    println!(
        r#"
Deploy Record Examples:

# Add a successful deployment
i-rs-deploy add myapp production v1.2.3 --status success --tag frontend --remark "New login feature"

# Add a failed deployment
i-rs-deploy add myapp staging v1.2.4 --status failed --tag backend --remark "Database connection timeout"

# List all deployments
i-rs-deploy list

# List deployments for a specific project
i-rs-deploy list --project myapp

# List deployments for a specific environment
i-rs-deploy list --environment production

# Filter by tag
i-rs-deploy list --tag frontend

# Get deployment details
i-rs-deploy get abc12345

# Delete a deployment record
i-rs-deploy delete abc12345

# Rollback to previous version
i-rs-deploy rollback myapp production

# Rollback to specific version
i-rs-deploy rollback myapp production --rollback-to abc12345

# Show deployment statistics
i-rs-deploy stats

# Stats for specific project
i-rs-deploy stats --project myapp

# Stats for specific environment
i-rs-deploy stats --environment production

# JSON output
i-rs-deploy list --json
i-rs-deploy get abc12345 --json
"#
    );
}
