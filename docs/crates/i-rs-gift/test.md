# i-rs-gift Test Records

## Test Data

### Gifts Added

```bash
# Test 1: Add a birthday gift
i-rs-gift add "Birthday Watch" sent "Mom" birthday 500 2024-12-25 --tag family

# Test 2: Add a Christmas gift received
i-rs-gift add "AirPods Pro" received "Boss" christmas 1200 2024-12-25 --tag work

# Test 3: Add an anniversary gift
i-rs-gift add "Rose Bouquet" sent "Wife" anniversary 300 2024-02-14 --tag romance

# Test 4: Add a Valentine's gift
i-rs-gift add "Diamond Ring" sent "Wife" valentine 5000 2024-02-14 --tag romance --tag special

# Test 5: Add a graduation gift
i-rs-gift add "Laptop" sent "Nephew" graduation 3000 2024-06-15 --tag family --tag education
```

### List Operations

```bash
# List all gifts
i-rs-gift list

# List only sent gifts
i-rs-gift list --type sent

# List only received gifts
i-rs-gift list --type received

# List gifts by tag
i-rs-gift list --tag family
```

### Get Operation

```bash
# Get gift details
i-rs-gift get "Birthday Watch"
```

### Statistics

```bash
# View statistics
i-rs-gift stats
```

## Expected Results

### List Output

| NAME | TYPE | RECIPIENT | OCCASION | VALUE |
|------|------|-----------|----------|-------|
| Birthday Watch | sent | Mom | birthday | 500.00 |
| AirPods Pro | received | Boss | christmas | 1200.00 |
| Rose Bouquet | sent | Wife | anniversary | 300.00 |
| Diamond Ring | sent | Wife | valentine | 5000.00 |
| Laptop | sent | Nephew | graduation | 3000.00 |

### Stats Output

```
Overview:
  Total: 5
  Sent: 4
  Received: 1

Value Summary:
  Total Sent: 8800.00
  Total Received: 1200.00
  Balance: -7600.00

By Occasion:
  birthday (🎂): 1
  christmas (🎄): 1
  anniversary (💕): 1
  valentine (❤️): 1
  graduation (🎓): 1

Top Recipients:
  Wife: 2 gifts
  Mom: 1 gifts
  Boss: 1 gifts
  Nephew: 1 gifts
```
