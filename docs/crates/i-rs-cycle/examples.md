# i-rs-cycle Examples

## Basic Usage

### Recording Events

```bash
# Period start
i-rs-cycle add 2024-01-15 period

# Period end or spotting
i-rs-cycle add 2024-01-20 spotting

# Ovulation
i-rs-cycle add 2024-01-28 ovulation

# Fertile window
i-rs-cycle add 2024-01-26 fertile
```

### With Symptoms

```bash
# Period with symptoms
i-rs-cycle add 2024-01-15 period --symptom cramps --symptom headache

# Spotting with symptoms
i-rs-cycle add 2024-01-20 spotting --symptom bloating
```

### With Tags

```bash
# Tagged entries
i-rs-cycle add 2024-01-15 period --tag heavy --tag first-day
i-rs-cycle add 2024-01-20 spotting --tag light --tag last-day
```

## Viewing Records

```bash
# List all records
i-rs-cycle list

# Filter by tag
i-rs-cycle list --tag heavy
```

## Managing Records

```bash
# Get record details
i-rs-cycle get abc12345

# Delete a record
i-rs-cycle delete abc12345
```

## Cycle Tracking

```bash
# Track a full cycle
i-rs-cycle add 2024-01-01 period --tag cycle-start --symptom cramps
i-rs-cycle add 2024-01-05 period --tag cycle-end
i-rs-cycle add 2024-01-14 ovulation --symptom mittelschmerz
i-rs-cycle add 2024-01-29 period --tag cycle-start --symptom cramps --symptom headache
```