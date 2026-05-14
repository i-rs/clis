# i-rs-goal

Savings goal tracking CLI tool for personal finance management.

## Features

- Create and track multiple savings goals
- Set target amounts and deadlines
- Record deposits and track progress
- Create milestones for intermediate targets
- Tag support for organization
- Progress percentage calculation
- Statistics overview

## Quick Start

```bash
# Create a savings goal
i-rs-goal add "Emergency Fund" --target 10000 --deadline 2025-12-31

# Deposit to a goal
i-rs-goal deposit "Emergency Fund" --amount 500

# List all goals
i-rs-goal list

# View goal details
i-rs-goal get "Emergency Fund"
```

## Data Storage

- Config: `~/.config/i-rs/goal.json`

## Commands Overview

| Command | Description |
|---------|-------------|
| `add` | Create a new savings goal |
| `list` | List all savings goals |
| `get` | View goal details |
| `update` | Update goal properties |
| `delete` | Delete a goal |
| `deposit` | Deposit to a goal |
| `milestone` | Manage milestones |
| `stats` | View statistics |
| `example` | Show usage examples |
| `skill` | Show skill documentation |
