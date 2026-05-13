# i-rs-domain Examples

## Basic Usage

### Adding Domains

```bash
# Add domain with registrar and password
i-rs-domain add example.com 2025-12-31 --registrar GoDaddy --password ghp_xxxxx --tag important --tag primary

# Add domain without password
i-rs-domain add mysite.io 2026-03-15 --registrar Namecheap --tag personal

# Add domain with just tags
i-rs-domain add blog.net 2025-06-01 --tag blog --tag personal

# Add multiple domains at once
i-rs-domain add company.com 2025-09-30 --registrar Cloudflare --password cf_pass --tag production --tag company
i-rs-domain add shop.io 2025-11-15 --registrar Shopify --password shp_xxx --tag ecommerce
i-rs-domain add api.org 2026-01-01 --tag api --tag backend
```

### Managing Domains

```bash
# List all domains
i-rs-domain list

# List important domains
i-rs-domain list --tag important

# List production domains
i-rs-domain list --tag production

# Get domain details
i-rs-domain get example.com

# Get with registrar password
i-rs-domain get example.com --show-password
```

### Updating Domains

```bash
# Update expiry date
i-rs-domain update example.com --expiry-date 2026-12-31

# Update registrar
i-rs-domain update example.com --registrar Cloudflare

# Update password
i-rs-domain update example.com --password new_password

# Update tags
i-rs-domain update example.com --tag critical --tag production

# Full update
i-rs-domain update example.com -e 2027-01-01 -r Cloudflare -p newpass -t critical
```

### Deleting Domains

```bash
i-rs-domain delete old-domain.com
i-rs-domain delete test-domain.io
```

## Organized Domain Management

### By Priority

```bash
# Critical domains
i-rs-domain add main-site.com 2025-06-01 --registrar Cloudflare --tag critical --tag production
i-rs-domain add api.com 2025-07-01 --registrar Cloudflare --tag critical --tag api

# Important domains
i-rs-domain add blog.com 2025-09-01 --registrar GoDaddy --tag important --tag content
i-rs-domain add docs.com 2025-10-01 --registrar Namecheap --tag important --tag docs

# Personal domains
i-rs-domain add portfolio.io 2026-01-01 --tag personal
i-rs-domain add projects.me 2026-02-01 --tag personal --tag projects
```

### By Registrar

```bash
# Cloudflare domains
i-rs-domain add cf-domain1.com 2025-12-01 --registrar Cloudflare --password cf_xxx --tag cloudflare
i-rs-domain add cf-domain2.io 2025-12-01 --registrar Cloudflare --password cf_xxx --tag cloudflare

# GoDaddy domains
i-rs-domain add gd-domain.com 2025-06-01 --registrar GoDaddy --password gd_xxx --tag godaddy
```

## Real-World Scenarios

### Domain Portfolio Management

```bash
# Add all your domains
i-rs-domain add myapp.com 2025-12-01 --registrar Cloudflare --password xxx --tag primary --tag production
i-rs-domain add myapp.io 2026-01-15 --registrar Namecheap --tag primary --tag brand
i-rs-domain add myapp.dev 2026-02-01 --registrar Google --tag primary --tag dev

# Add subdomains as separate entries
i-rs-domain add api.myapp.com 2025-12-01 --registrar Cloudflare --tag api --tag backend

# Review all domains annually
i-rs-domain list --tag primary
```

### Renewal Reminders

```bash
# Track renewal dates
i-rs-domain add important.com 2025-03-15 --registrar GoDaddy --tag renewal-2025 --tag critical
i-rs-domain add domain.net 2025-04-20 --registrar Namecheap --tag renewal-2025 --tag important

# Use list to check upcoming renewals
i-rs-domain list

# Filter by expiry timing (via tags)
i-rs-domain add expiring-soon.com 2025-02-01 --tag needs-renewal
```

### Corporate Domain Management

```bash
# Add company domains
i-rs-domain add company.com 2026-01-01 --registrar Cloudflare --password xxx --tag corporate --tag primary
i-rs-domain add company.net 2026-01-01 --registrar Cloudflare --password xxx --tag corporate --tag brand
i-rs-domain add company.org 2026-01-01 --registrar Cloudflare --password xxx --tag corporate --tag brand
i-rs-domain add product.io 2025-09-01 --registrar AWS --password xxx --tag corporate --tag product

# List corporate domains
i-rs-domain list --tag corporate
```