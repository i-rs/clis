# i-rs-contact Test Records

## Test Cases

### 1. Add Contact

**Command:**
```bash
i-rs-contact add test_contact --phone 13800138000 --email test@example.com --relationship friend --tag test
```

**Expected Result:**
- Contact created successfully
- Store updated with new contact

### 2. List Contacts

**Command:**
```bash
i-rs-contact list
```

**Expected Result:**
- Table with all contacts displayed
- Contact details visible

### 3. List with Tag Filter

**Command:**
```bash
i-rs-contact list --tag test
```

**Expected Result:**
- Only contacts with "test" tag displayed

### 4. Get Contact

**Command:**
```bash
i-rs-contact get test_contact
```

**Expected Result:**
- Detailed contact information displayed
- Phone, email, relationship shown

### 5. Update Contact

**Command:**
```bash
i-rs-contact update test_contact --phone 13900139000 --relationship colleague
```

**Expected Result:**
- Contact phone updated
- Contact relationship updated
- Updated_at timestamp changed

### 6. Stats

**Command:**
```bash
i-rs-contact stats
```

**Expected Result:**
- Total contact count displayed
- By relationship breakdown
- By tag breakdown
- Contacts needing reminder

### 7. Remind

**Command:**
```bash
i-rs-contact remind
```

**Expected Result:**
- List of contacts not contacted in 30+ days
- Days since last contact shown

### 8. Delete Contact

**Command:**
```bash
i-rs-contact delete test_contact
```

**Expected Result:**
- Contact removed from store
- Confirmation message displayed

### 9. JSON Output

**Command:**
```bash
i-rs-contact list --json
```

**Expected Result:**
- Valid JSON output
- Contains success, data, and meta fields

## Edge Cases

### 10. Non-existent Contact

**Command:**
```bash
i-rs-contact get nonexistent_contact
```

**Expected Result:**
- Error message: "Contact 'nonexistent_contact' not found"

### 11. Duplicate Contact

**Command:**
```bash
i-rs-contact add test_contact
i-rs-contact add test_contact
```

**Expected Result:**
- Second command fails with "Contact 'test_contact' already exists"

### 12. Empty List

**Command:**
```bash
i-rs-contact list
```

**Expected Result:**
- Message: "No contacts found."

### 13. Custom Reminder Days

**Command:**
```bash
i-rs-contact remind --days 7
```

**Expected Result:**
- List of contacts not contacted in 7+ days

## Test Data

```json
{
  "entries": {
    "test_contact": {
      "name": "test_contact",
      "phone": "13800138000",
      "email": "test@example.com",
      "relationship": "friend",
      "tags": ["test"],
      "remark": [],
      "last_contact": null,
      "contact_count": 0,
      "created_at": 1705315200,
      "updated_at": 1705315200
    }
  }
}
```
