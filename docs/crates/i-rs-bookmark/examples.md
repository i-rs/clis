# i-rs-bookmark Examples

## Basic Usage

### Adding Bookmarks

```bash
# Simple bookmark (no credentials)
i-rs-bookmark add wikipedia https://wikipedia.org --tag reference --tag wiki

# Bookmark with account only
i-rs-bookmark add twitter https://twitter.com --account user --tag social

# Bookmark with full credentials
i-rs-bookmark add aws-console https://aws.amazon.com --account admin@example.com --password secret123 --tag cloud --tag aws

# Bookmark with multiple tags
i-rs-bookmark add stackoverflow https://stackoverflow.com --account user@email.com --password pass --tag dev --tag qa --tag reference
```

### Managing Bookmarks

```bash
# List all bookmarks
i-rs-bookmark list

# List by category
i-rs-bookmark list --tag dev
i-rs-bookmark list --tag cloud

# Get bookmark details
i-rs-bookmark get aws-console

# Get with password
i-rs-bookmark get aws-console --show-password
```

### Updating Bookmarks

```bash
# Update URL
i-rs-bookmark update aws-console --url https://console.aws.amazon.com

# Update credentials
i-rs-bookmark update aws-console --password newpassword

# Update tags
i-rs-bookmark update aws-console --tag production --tag important
```

### Deleting Bookmarks

```bash
i-rs-bookmark delete old-bookmark
```

## Organized Bookmarks

### By Project

```bash
# Project Alpha
i-rs-bookmark add alpha-docs https://docs.alpha.example.com --tag alpha --tag docs
i-rs-bookmark add alpha-repo https://github.com/company/alpha --tag alpha --tag repo

# Project Beta
i-rs-bookmark add beta-docs https://docs.beta.example.com --tag beta --tag docs
i-rs-bookmark add beta-jira https://company.atlassian.net --tag beta --tag tickets
```

### By Technology

```bash
# Docker
i-rs-bookmark add docker-hub https://hub.docker.com --account user --tag docker --tag registry
i-rs-bookmark add docker-docs https://docs.docker.com --tag docker --tag docs

# Kubernetes
i-rs-bookmark add k8s-docs https://kubernetes.io/docs/ --tag kubernetes --tag docs
i-rs-bookmark add k8s-dashboard https://dashboard.k8s.io --tag kubernetes --tag ops

# AWS
i-rs-bookmark add aws-ec2 https://console.aws.amazon.com/ec2 --tag aws --tag compute
i-rs-bookmark add aws-s3 https://console.aws.amazon.com/s3 --tag aws --tag storage
```