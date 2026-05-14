# i-rs-grocery Test Records

## Test Cases

### 1. Add Item

**Command:**
```bash
i-rs-grocery add milk 2 bottles --tag dairy
```

**Expected Result:**
- Item added successfully
- Store updated with new item

### 2. List Items

**Command:**
```bash
i-rs-grocery list
```

**Expected Result:**
- Table with all items displayed
- Status shows "🔄 Needed"

### 3. Purchase Item

**Command:**
```bash
i-rs-grocery purchase milk
```

**Expected Result:**
- Status changes to "✅ Purchased"
- Updated_at timestamp changed

### 4. Filter Items

**Command:**
```bash
i-rs-grocery list --tag dairy
i-rs-grocery list --purchased
i-rs-grocery list --needed
```

**Expected Result:**
- Correct items filtered

### 5. Clear Purchased

**Command:**
```bash
i-rs-grocery clear
```

**Expected Result:**
- All purchased items removed
- Only needed items remain

### 6. Get Item

**Command:**
```bash
i-rs-grocery get milk
```

**Expected Result:**
- Detailed item information displayed

### 7. Update Item

**Command:**
```bash
i-rs-grocery update milk --quantity 3
```

**Expected Result:**
- Quantity updated to 3
- Updated_at timestamp changed

### 8. Delete Item

**Command:**
```bash
i-rs-grocery delete milk
```

**Expected Result:**
- Item removed from store
- Confirmation message displayed

### 9. JSON Output

**Command:**
```bash
i-rs-grocery list --json
```

**Expected Result:**
- Valid JSON output
- Contains success, data, and meta fields

## Edge Cases

### 10. Duplicate Item

**Command:**
```bash
i-rs-grocery add milk 2 bottles
i-rs-grocery add milk 1 bottle
```

**Expected Result:**
- Second command fails with "Item 'milk' already exists"

### 11. Non-existent Item

**Command:**
```bash
i-rs-grocery get nonexistent
```

**Expected Result:**
- Error: "Item 'nonexistent' not found"

## Test Data

```json
{
  "entries": {
    "milk": {
      "name": "milk",
      "quantity": 2,
      "unit": "bottles",
      "purchased": false,
      "tags": ["dairy"],
      "remark": [],
      "created_at": 1705315200,
      "updated_at": 1705315200
    }
  }
}
```