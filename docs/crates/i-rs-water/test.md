# i-rs-water Test Records

## Test Data

```bash
# Basic water intake
i-rs-water add 250
i-rs-water add 500
i-rs-water add 750

# With tags
i-rs-water add 300 --tag morning
i-rs-water add 400 --tag afternoon
i-rs-water add 300 --tag evening

# With remarks
i-rs-water add 500 --remark after workout
i-rs-water add 250 --remark before breakfast
```