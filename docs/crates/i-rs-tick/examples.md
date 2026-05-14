# i-rs-tick Examples

## Basic Usage

### Recording Durations

```bash
# Quick recording (duration in seconds)
i-rs-tick add "Meeting" --duration 3600
i-rs-tick add "Coding" --duration 7200
i-rs-tick add "Email" --duration 1800

# Converting hours/minutes
# 1 hour = 3600 seconds
# 30 minutes = 1800 seconds
# 15 minutes = 900 seconds
```

### With Descriptions

```bash
# Add context
i-rs-tick add "Meeting" --duration 3600 --description "Sprint planning"
i-rs-tick add "Coding" --duration 7200 --description "Feature implementation"
i-rs-tick add "Code Review" --duration 1800 --description "PR #123"
```

## Viewing Records

```bash
# List all
i-rs-tick list

# Get details
i-rs-tick get abc12345
```

## Project Tracking

```bash
# Development tasks
i-rs-tick add "Frontend" --duration 14400 --tag project --tag frontend --remark "Dashboard UI"  # 4 hours
i-rs-tick add "Backend API" --duration 10800 --tag project --tag backend --remark "REST endpoints"  # 3 hours
i-rs-tick add "Database" --duration 7200 --tag project --tag backend --remark "Schema design"  # 2 hours

# Documentation
i-rs-tick add "Write Docs" --duration 3600 --tag project --tag docs  # 1 hour
i-rs-tick add "Update README" --duration 1800 --tag project --tag docs  # 30 minutes
```

## Daily Time Log

```bash
# Morning block
i-rs-tick add "Standup" --duration 900 --tag daily --tag meeting  # 15 min
i-rs-tick add "Deep Work" --duration 7200 --tag daily --tag focused  # 2 hours
i-rs-tick add "Code Review" --duration 3600 --tag daily --tag collaboration  # 1 hour

# Afternoon block
i-rs-tick add "Lunch Break" --duration 3600 --tag break  # 1 hour
i-rs-tick add "Feature Development" --duration 10800 --tag daily --tag coding  # 3 hours
i-rs-tick add "Bug Fixes" --duration 5400 --tag daily --tag coding  # 1.5 hours
```

## Time Estimation

```bash
# Track similar tasks to estimate future work
i-rs-tick add "Write Tests" --duration 3600 --tag estimation --tag testing
i-rs-tick add "Write Tests" --duration 2700 --tag estimation --tag testing  # Next time estimate
i-rs-tick add "Write Tests" --duration 3000 --tag estimation --tag testing  # Average: ~1 hour
```