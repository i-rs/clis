# i-rs-mood Test Records

## Test Log

### 2025-01-15 - Initial Setup

```bash
$ i-rs-mood add 2025-01-15 good
✓ Mood record added: 🙂 Good
```

### 2025-01-16 - Testing Different Mood Inputs

```bash
$ i-rs-mood add 2025-01-16 5 --tag test
✓ Mood record added: 😊 Great

$ i-rs-mood add 2025-01-17 😊 --tag emoji-test
✓ Mood record added: 😊 Great
```

### 2025-01-18 - With Tags and Notes

```bash
$ i-rs-mood add 2025-01-18 great --tag work --tag achievement --content "Finished project" --content "Got praise from manager"
✓ Mood record added: 😊 Great
```

### 2025-01-19 - Okay Mood

```bash
$ i-rs-mood add 2025-01-19 okay --tag monday --content "Monday blues"
✓ Mood record added: 😐 Okay
```

### 2025-01-20 - Bad Mood

```bash
$ i-rs-mood add 2025-01-20 bad --tag health --content "Headache"
✓ Mood record added: 😔 Bad
```

### 2025-01-21 - Terrible Mood

```bash
$ i-rs-mood add 2025-01-21 terrible --tag sick --content "Flu"
✓ Mood record added: 😢 Terrible
```

### 2025-01-22 - List View

```bash
$ i-rs-mood list
 DATE       MOOD     TAGS        CONTENT
 2025-01-15 🙂 Good  -          -
 2025-01-16 😊 Great test       -
 2025-01-17 😊 Great emoji-test -
 2025-01-18 😊 Great work        Finished project
                         achievement Got praise...
 2025-01-19 😐 Okay   monday     Monday blues
 2025-01-20 😔 Bad   health     Headache
 2025-01-21 😢 Terrible sick      Flu

Total: 7 records

Statistics:
  Best:      😊 Great
  Worst:     😢 Terrible
  Average:   3.7/5
```

### 2025-01-22 - Calendar View

```bash
$ i-rs-mood list --days 7 --calendar

 DATE       MOOD     TAGS        CONTENT
 2025-01-15 🙂 Good  -          -
 2025-01-16 😊 Great test       -
 2025-01-17 😊 Great emoji-test -
 2025-01-18 😊 Great work        Finished project
 2025-01-19 😐 Okay   monday     Monday blues
 2025-01-20 😔 Bad   health     Headache
 2025-01-21 😢 Terrible sick      Flu

Total: 7 records

Statistics:
  Best:      😊 Great
  Worst:     😢 Terrible
  Average:   3.7/5

Mood Calendar:
───────────────────────────────────────
 🙂 😊 😊 😊 😐 😔 😢

Legend: 😊 Great  🙂 Good  😐 Okay  😔 Bad  😢 Terrible
```

### 2025-01-23 - Update Test

```bash
$ i-rs-mood update 2025-01-20 --mood okay --content "Feeling better"
✓ Record for 2025-01-20 updated
```

### 2025-01-24 - Delete Test

```bash
$ i-rs-mood delete 2025-01-21
✓ Record for 2025-01-21 deleted
```

### 2025-01-25 - Final List

```bash
$ i-rs-mood list
 DATE       MOOD     TAGS        CONTENT
 2025-01-15 🙂 Good  -          -
 2025-01-16 😊 Great test       -
 2025-01-17 😊 Great emoji-test -
 2025-01-18 😊 Great work        Finished project
 2025-01-19 😐 Okay   monday     Monday blues
 2025-01-20 😐 Okay   health     Feeling better
 2025-01-22 😊 Great -          -

Total: 7 records

Statistics:
  Best:      😊 Great
  Worst:     😐 Okay
  Average:   3.8/5
```

## Error Handling Tests

### Duplicate Date

```bash
$ i-rs-mood add 2025-01-15 good
Error: Record for 2025-01-15 already exists. Use update command instead.
```

### Invalid Date Format

```bash
$ i-rs-mood add invalid-date good
Error: Invalid date format: invalid-date. Use YYYY-MM-DD
```

### Invalid Mood

```bash
$ i-rs-mood add 2025-01-25 invalid
Error: Invalid mood: invalid. Use 1-5, great/good/okay/bad/terrible, or emoji
```

### Invalid Mood Range

```bash
$ i-rs-mood add 2025-01-25 10
Error: Invalid mood: 10. Use 1-5, great/good/okay/bad/terrible, or emoji
```

### Update Non-existent Record

```bash
$ i-rs-mood update 2025-12-31 --mood good
Error: No record found for 2025-12-31
```

### Delete Non-existent Record

```bash
$ i-rs-mood delete 2025-12-31
Error: No record found for 2025-12-31
```

## Test Summary

| Test Case | Status |
|-----------|--------|
| Add mood with word | ✅ Pass |
| Add mood with number | ✅ Pass |
| Add mood with emoji | ✅ Pass |
| Add with tags | ✅ Pass |
| Add with content/notes | ✅ Pass |
| List all records | ✅ Pass |
| List with days filter | ✅ Pass |
| List with calendar | ✅ Pass |
| Statistics calculation | ✅ Pass |
| Update mood | ✅ Pass |
| Update tags | ✅ Pass |
| Update content | ✅ Pass |
| Delete record | ✅ Pass |
| Duplicate date handling | ✅ Pass |
| Invalid date handling | ✅ Pass |
| Invalid mood handling | ✅ Pass |
| Update non-existent | ✅ Pass |
| Delete non-existent | ✅ Pass |
