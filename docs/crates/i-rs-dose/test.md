# i-rs-dose Test Records

## Test Data

```bash
# Basic supplements
i-rs-dose add "Vitamin D" --dosage 1000 --unit IU
i-rs-dose add "Vitamin C" --dosage 500 --unit mg
i-rs-dose add "Vitamin B12" --dosage 1000 --unit mcg

# Medications
i-rs-dose add "Ibuprofen" --dosage 400 --unit mg
i-rs-dose add "Aspirin" --dosage 100 --unit mg

# With tags
i-rs-dose add "Fish Oil" --dosage 1000 --unit mg --tag morning
i-rs-dose add "Calcium" --dosage 600 --unit mg --tag evening
```