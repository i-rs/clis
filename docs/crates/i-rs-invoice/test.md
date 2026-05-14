# i-rs-invoice Test Records

## Test 1: Add Electronic Invoice

```bash
i-rs-invoice add "Office Supplies" --amount 299.99 --type electronic --tags expense --tags office
```

Expected output:
```
Invoice added successfully
```

## Test 2: Add Paper Invoice

```bash
i-rs-invoice add "Travel Expense" --amount 1500.00 --date 2024-01-15 --type paper --tags travel
```

Expected output:
```
Invoice added successfully
```

## Test 3: List All Invoices

```bash
i-rs-invoice list
```

Expected: Table with invoice details

## Test 4: List Unreimbursed

```bash
i-rs-invoice list --unreimbursed
```

Expected: Only unreimbursed invoices shown

## Test 5: View Statistics

```bash
i-rs-invoice stats
```

Expected output:
```
=== Invoice Statistics ===
Total: <amount>
Reimbursed: <amount>
Unreimbursed: <amount>
```

## Test 6: Update Invoice

```bash
i-rs-invoice update <id> --reimbursed
```

Expected output:
```
Invoice '<id>' updated successfully
```

## Test 7: JSON Output

```bash
i-rs-invoice list --json
```

Expected: Valid JSON response
