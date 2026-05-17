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
- JSON output for scripting

## Install

```bash
npm install -g @i-rs/i-rs-goal
# or
brew install i-rs/homebrew-tap/i-rs-goal
```

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

# Add a milestone
i-rs-goal milestone -g "Emergency Fund" -n "First 1000" -a 1000

# View statistics
i-rs-goal stats
```

## Commands

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

## Data Storage

- macOS: `~/.config/i-rs/goal.json`
- Linux: `~/.config/i-rs/goal.json`
- Windows: `~\AppData\Roaming\i-rs\goal.json`

## License

MIT OR Apache-2.0
