# i-rs-allergy Examples

## Basic Usage

### Recording Reactions

```bash
# Food allergies
i-rs-allergy add "Peanuts" mild --symptom hives
i-rs-allergy add "Shellfish" moderate --symptom swelling --symptom nausea

# Environmental allergies
i-rs-allergy add "Pollen" mild --symptom sneezing --symptom watery-eyes
i-rs-allergy add "Dust" moderate --symptom itching --symptom sneezing
```

### With Tags

```bash
# Tagged entries
i-rs-allergy add "Peanuts" severe --tag food --symptom anaphylaxis
i-rs-allergy add "Cats" mild --tag pet --symptom sneezing
```

## Viewing Records

```bash
# List all records
i-rs-allergy list

# Filter by tag
i-rs-allergy list --tag food
i-rs-allergy list --tag pet
```

## Managing Records

```bash
# Get record details
i-rs-allergy get abc12345

# Delete a record
i-rs-allergy delete abc12345
```

## Allergy Journal

```bash
# Track a reaction event
i-rs-allergy add "Dairy" moderate --symptom bloating --symptom headache --tag food --remark "Ate ice cream"

# Environmental reaction
i-rs-allergy add "Mold" severe --symptom coughing --symptom shortness-of-breath --tag environmental --remark "Bathroom mold exposure"
```