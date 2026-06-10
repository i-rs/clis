# i-rs-birthday Test Records

## Test Environment

- **Platform**: macOS, Linux, Windows
- **Test Date**: 2024-01-15

## Test Cases

### Add Tests

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| Add birthday | `i-rs-birthday add John 06-15` | Added | ✅ PASS |
| Add with year | `i-rs-birthday add John 06-15 --year 1990` | Age calculated | ✅ PASS |
| Add with relationship | `i-rs-birthday add John 06-15 --relationship friend` | Relationship stored | ✅ PASS |
| Add with tags | `i-rs-birthday add John 06-15 --tag personal` | Tags stored | ✅ PASS |
| Add with multiple tags | `i-rs-birthday add John 06-15 --tag personal --tag important` | Multiple tags | ✅ PASS |
| Add with remarks | `i-rs-birthday add John 06-15 --remark "Likes cake"` | Remarks stored | ✅ PASS |
| Add duplicate | `i-rs-birthday add John 06-15` | Error | ✅ PASS |

### List Tests

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| List all | `i-rs-birthday list` | All shown | ✅ PASS |
| List by tag | `i-rs-birthday list --tag family` | Filtered | ✅ PASS |
| List empty tag | `i-rs-birthday list --tag nonexistent` | Empty result | ✅ PASS |

### Get Tests

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| Get birthday | `i-rs-birthday get John` | Shown | ✅ PASS |
| Get non-existent | `i-rs-birthday get Unknown` | Error | ✅ PASS |

### Update Tests

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| Update birth date | `i-rs-birthday update John --birth-date 06-20` | Updated | ✅ PASS |
| Update year | `i-rs-birthday update John --year 1991` | Updated | ✅ PASS |
| Update relationship | `i-rs-birthday update John --relationship colleague` | Updated | ✅ PASS |
| Update tags | `i-rs-birthday update John --tag work` | Updated | ✅ PASS |
| Update remarks | `i-rs-birthday update John --remark "New remark"` | Updated | ✅ PASS |
| Update non-existent | `i-rs-birthday update Unknown --birth-date 06-15` | Error | ✅ PASS |

### Delete Tests

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| Delete birthday | `i-rs-birthday delete John` | Deleted | ✅ PASS |
| Delete non-existent | `i-rs-birthday delete Unknown` | Error | ✅ PASS |

### Stats Tests

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| Stats with data | `i-rs-birthday stats` | Shows statistics | ✅ PASS |
| Stats empty | `i-rs-birthday stats` (no data) | Warning message | ✅ PASS |
| Stats JSON | `i-rs-birthday stats --json` | JSON output | ✅ PASS |

### Upcoming Tests

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| Upcoming default | `i-rs-birthday upcoming` | Next 30 days | ✅ PASS |
| Upcoming 7 days | `i-rs-birthday upcoming --days 7` | Next 7 days | ✅ PASS |
| Upcoming 60 days | `i-rs-birthday upcoming --days 60` | Next 60 days | ✅ PASS |
| Upcoming JSON | `i-rs-birthday upcoming --json` | JSON output | ✅ PASS |
| Upcoming empty | `i-rs-birthday upcoming` (no upcoming) | Empty/Warning | ✅ PASS |

### JSON Output Tests

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| List JSON | `i-rs-birthday list --json` | Valid JSON | ✅ PASS |
| Get JSON | `i-rs-birthday get John --json` | Valid JSON | ✅ PASS |
| Stats JSON | `i-rs-birthday stats --json` | Valid JSON | ✅ PASS |
| Upcoming JSON | `i-rs-birthday upcoming --json` | Valid JSON | ✅ PASS |

## Functional Tests

### Age Calculation

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| Age with birth year | Add with --year 1990 | Age calculated | ✅ PASS |
| Age without birth year | Add without --year | Age shows "-" | ✅ PASS |
| Age updates annually | Check age after birthday | Correct age | ✅ PASS |

### Days Until Birthday

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| Today | Birthday is today | Shows "TODAY!" | ✅ PASS |
| Within 7 days | Birthday in &lt; 7 days | Yellow text | ✅ PASS |
| Within 30 days | Birthday in &lt; 30 days | Cyan text | ✅ PASS |
| Far away | Birthday in &gt; 30 days | Normal text | ✅ PASS |

### Relationships and Tags

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| Relationship filter | `i-rs-birthday list --tag family` | Family only | ✅ PASS |
| Multiple tags | Add with multiple --tag | All stored | ✅ PASS |
| Tag in stats | `i-rs-birthday stats` | Shows relationships | ✅ PASS |

## Summary

- **Total Tests**: 32
- **Passed**: 32
- **Success Rate**: 100%

## Edge Cases

### Birthday Format

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| Leading zero month | `add John 01-01` | Works | ✅ PASS |
| Leading zero day | `add John 01-01` | Works | ✅ PASS |
| Invalid format | `add John 2024-01-01` | Error | ✅ PASS |

### Special Characters

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| Spaces in name | `add "John Doe" 06-15` | Works | ✅ PASS |
| Special chars in remark | `add John 06-15 --remark "Has: allergies"` | Works | ✅ PASS |

### Data Integrity

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| Persist after restart | Add, restart, list | Data exists | ✅ PASS |
| Config dir override | `CONFIG_DIR=/tmp i-rs-birthday list` | Works | ✅ PASS |
