# i-rs-sleep Test Records

## Test Cases

### 1. Add Sleep Record

**Command:**
```bash
i-rs-sleep add 22:30 06:45 4 --tag test
```

**Expected Result:**
- Record created successfully
- Duration calculated correctly (8.25 hours)
- Store updated

### 2. List Records

**Command:**
```bash
i-rs-sleep list
```

**Expected Result:**
- Table with all records displayed
- Duration formatted correctly

### 3. Get Record

**Command:**
```bash
i-rs-sleep get <ID>
```

**Expected Result:**
- Detailed record information displayed

### 4. Statistics

**Command:**
```bash
i-rs-sleep stats
```

**Expected Result:**
- Average duration calculated
- Average quality calculated
- Min/max duration shown

### 5. Update Record

**Command:**
```bash
i-rs-sleep update <ID> --quality 5
```

**Expected Result:**
- Quality updated to 5
- Updated_at timestamp changed

### 6. Delete Record

**Command:**
```bash
i-rs-sleep delete <ID>
```

**Expected Result:**
- Record removed from store
- Confirmation message displayed

### 7. JSON Output

**Command:**
```bash
i-rs-sleep list --json
```

**Expected Result:**
- Valid JSON output
- Contains success, data, and meta fields

## Edge Cases

### 8. Invalid Quality

**Command:**
```bash
i-rs-sleep add 22:30 06:45 6 --tag test
```

**Expected Result:**
- Error: "Quality must be between 1 and 5"

### 9. Invalid Time Format

**Command:**
```bash
i-rs-sleep add 10:30 PM 6:45 AM 4
```

**Expected Result:**
- Error: "Invalid time format. Use HH:MM"

## Test Data

```json
{
  "entries": {
    "test-id-1": {
      "id": "test-id-1",
      "bedtime": 1705313400,
      "wake_time": 1705342800,
      "quality": 4,
      "tags": ["test"],
      "remark": [],
      "created_at": 1705313400,
      "updated_at": 1705313400
    }
  }
}
```