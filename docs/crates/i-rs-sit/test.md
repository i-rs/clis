# i-rs-sit Test Records

## Test Data

```bash
# Basic sitting durations
i-rs-sit add 30
i-rs-sit add 60
i-rs-sit add 90

# With tags
i-rs-sit add 60 --tag work
i-rs-sit add 120 --tag office
i-rs-sit add 45 --tag home

# With remarks
i-rs-sit add 90 --remark during meeting
i-rs-sit add 60 --remark after lunch
```