# i-rs-goal Usage

## Global Flags

- `--json` — Output in JSON format

## add

Create a new savings goal.

```bash
i-rs-goal add <name> --target <amount> --deadline <date> [options]
```

**Options:**
- `-t, --target <amount>` - Target amount to save (required)
- `-d, --deadline <date>` - Deadline (YYYY-MM-DD format, required)
- `--tags <tags>` - Tags (comma-separated)
- `--remark <remarks>` - Remarks (comma-separated)
- `-m, --milestones <milestones>` - Milestones as name:amount pairs (comma-separated)

**Example:**
```bash
i-rs-goal add "Emergency Fund" --target 10000 --deadline 2025-12-31 --tags emergency,finance
```

## list

List all savings goals.

```bash
i-rs-goal list [options]
```

**Options:**
- `--tag <tag>` - Filter by tag

**Example:**
```bash
i-rs-goal list
i-rs-goal list --tag emergency
```

## get

View goal details.

```bash
i-rs-goal get <name>
```

**Example:**
```bash
i-rs-goal get "Emergency Fund"
```

## update

Update goal properties.

```bash
i-rs-goal update <name> [options]
```

**Options:**
- `-t, --target <amount>` - New target amount
- `-d, --deadline <date>` - New deadline
- `--tags <tags>` - New tags (comma-separated)
- `--remark <remarks>` - New remarks (comma-separated)

**Example:**
```bash
i-rs-goal update "Emergency Fund" --target 15000
```

## delete

Delete a goal.

```bash
i-rs-goal delete <name>
```

**Example:**
```bash
i-rs-goal delete "Emergency Fund"
```

## deposit

Deposit to a goal.

```bash
i-rs-goal deposit <name> --amount <amount>
```

**Example:**
```bash
i-rs-goal deposit "Emergency Fund" --amount 500
```

## milestone

Manage milestones.

```bash
i-rs-goal milestone -g <goal> [options]
```

**Options:**
- `-g, --goal <name>` - Goal name (required)
- `-n, --name <name>` - Milestone name
- `-a, --amount <amount>` - Milestone amount
- `-l, --list` - List milestones only
- `-r, --remove <id>` - Remove milestone by ID

**Examples:**
```bash
# Add a milestone
i-rs-goal milestone -g "Emergency Fund" -n "First 1000" -a 1000

# List milestones
i-rs-goal milestone -g "Emergency Fund" --list

# Remove a milestone
i-rs-goal milestone -g "Emergency Fund" -r <milestone-id>
```

## stats

View statistics.

```bash
i-rs-goal stats [options]
```

**Options:**
- `--tag <tag>` - Filter by tag

**Example:**
```bash
i-rs-goal stats
i-rs-goal stats --tag emergency
```

## Global Options

- `--json` - Output in JSON format

**Example:**
```bash
i-rs-goal list --json
i-rs-goal get "Emergency Fund" --json
```

### data

Manage data (export, import, clear).

```bash
i-rs-goal data export
i-rs-goal data import [FILE]
i-rs-goal data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
### example

Show usage examples.

```bash
i-rs-goal example
```
### skill

Show skill information.

```bash
i-rs-goal skill [summary|content|raw]
```
