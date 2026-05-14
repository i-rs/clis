# i-rs-habit Test Records

## Test Cases

### 1. Create Habit

**Command:**
```bash
i-rs-habit add test_habit --description "Test habit" --frequency daily --tag test
```

**Expected Result:**
- Habit created successfully
- Store updated with new habit

### 2. Checkin Habit

**Command:**
```bash
i-rs-habit checkin test_habit
```

**Expected Result:**
- Checkin recorded
- Streak count increased

### 3. List Habits

**Command:**
```bash
i-rs-habit list
```

**Expected Result:**
- Table with all habits displayed
- Streak count visible

### 4. Get Habit

**Command:**
```bash
i-rs-habit get test_habit
```

**Expected Result:**
- Detailed habit information displayed
- Checkin history shown

### 5. Update Habit

**Command:**
```bash
i-rs-habit update test_habit --frequency weekly
```

**Expected Result:**
- Habit frequency updated
- Updated_at timestamp changed

### 6. Delete Habit

**Command:**
```bash
i-rs-habit delete test_habit
```

**Expected Result:**
- Habit removed from store
- Confirmation message displayed

### 7. JSON Output

**Command:**
```bash
i-rs-habit list --json
```

**Expected Result:**
- Valid JSON output
- Contains success, data, and meta fields

## Edge Cases

### 8. Non-existent Habit

**Command:**
```bash
i-rs-habit get nonexistent_habit
```

**Expected Result:**
- Error message: "Habit 'nonexistent_habit' not found"

### 9. Duplicate Habit

**Command:**
```bash
i-rs-habit add test_habit
i-rs-habit add test_habit
```

**Expected Result:**
- Second command fails with "Habit 'test_habit' already exists"

## Test Data

```json
{
  "entries": {
    "test_habit": {
      "name": "test_habit",
      "description": "Test habit",
      "frequency": "daily",
      "tags": ["test"],
      "remark": [],
      "checkins": [],
      "created_at": 1705315200,
      "updated_at": 1705315200
    }
  }
}
```