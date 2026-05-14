# i-rs-cycle Test Records

## Test Data

```bash
# Period events
i-rs-cycle add 2024-01-01 period
i-rs-cycle add 2024-01-05 period

# With symptoms
i-rs-cycle add 2024-01-01 period --symptom cramps --symptom headache
i-rs-cycle add 2024-01-05 spotting --symptom bloating

# Ovulation
i-rs-cycle add 2024-01-14 ovulation --symptom mittelschmerz

# Fertile window
i-rs-cycle add 2024-01-12 fertile
i-rs-cycle add 2024-01-13 fertile
```