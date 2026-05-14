# i-rs-sub Examples

## Basic Usage

### Adding Subscriptions

```bash
# Monthly subscriptions
i-rs-sub add "Netflix" --amount 15.99 --cycle monthly --next-date 2024-02-15
i-rs-sub add "Spotify" --amount 9.99 --cycle monthly --next-date 2024-02-20
i-rs-sub add "iCloud" --amount 6.99 --cycle monthly --next-date 2024-02-25

# Yearly subscriptions
i-rs-sub add "GitHub Pro" --amount 48 --cycle yearly --next-date 2024-06-01
i-rs-sub add "Adobe CC" --amount 599 --cycle yearly --next-date 2024-03-01
```

### With URLs

```bash
# With service URLs
i-rs-sub add "Netflix" --amount 15.99 --cycle monthly --next-date 2024-02-15 --url "https://netflix.com"
i-rs-sub add "Spotify" --amount 9.99 --cycle monthly --next-date 2024-02-20 --url "https://spotify.com"
```

## Viewing Subscriptions

```bash
# List all
i-rs-sub list

# Get details
i-rs-sub get Netflix
```

## Managing Subscriptions

```bash
# Update next billing date
i-rs-sub update "Netflix" --next-date 2024-03-15

# Update amount
i-rs-sub update "Spotify" --amount 10.99

# Delete
i-rs-sub delete "Old Service"
```

## Organized Tracking

```bash
# Entertainment
i-rs-sub add "Netflix" --amount 15.99 --cycle monthly --next-date 2024-02-15 --tag entertainment --tag streaming
i-rs-sub add "Spotify" --amount 9.99 --cycle monthly --next-date 2024-02-20 --tag entertainment --tag music
i-rs-sub add "Disney+" --amount 7.99 --cycle monthly --next-date 2024-02-10 --tag entertainment

# Productivity
i-rs-sub add "GitHub Pro" --amount 48 --cycle yearly --next-date 2024-06-01 --tag productivity --tag development
i-rs-sub add "Notion" --amount 10 --cycle monthly --next-date 2024-02-28 --tag productivity

# Cloud Storage
i-rs-sub add "iCloud" --amount 6.99 --cycle monthly --next-date 2024-02-25 --tag cloud
i-rs-sub add "Dropbox" --amount 11.99 --cycle monthly --next-date 2024-03-01 --tag cloud
```

## Budget Planning

```bash
# Track total monthly
# Entertainment
i-rs-sub add "Netflix" --amount 15.99 --cycle monthly --tag entertainment
i-rs-sub add "Spotify" --amount 9.99 --cycle monthly --tag entertainment
i-rs-sub add "YouTube Premium" --amount 13.99 --cycle monthly --tag entertainment

# Work tools
i-rs-sub add "Slack" --amount 8.75 --cycle monthly --tag work
i-rs-sub add "Zoom" --amount 15.99 --cycle monthly --tag work
i-rs-sub add "Notion" --amount 10 --cycle monthly --tag work
```