# i-rs-remind Test Records

## Test Environment

- **Platform**: macOS, Linux, Windows
- **Test Date**: 2024-01-15

## Test Cases

### Add Tests

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| Add with date | `i-rs-remind add test 2025-12-31` | Added | ✅ PASS |
| Add with date and time | `i-rs-remind add test 2025-12-31 14:00` | Added with time | ✅ PASS |
| Add with title | `i-rs-remind add test 2025-12-31 --title "Title"` | Title stored | ✅ PASS |
| Add with tags | `i-rs-remind add test 2025-12-31 --tag work --tag important` | Tags stored | ✅ PASS |
| Add duplicate | `i-rs-remind add test 2025-12-31` | Error | ✅ PASS |

### List Tests

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| List all | `i-rs-remind list` | All shown | ✅ PASS |
| Filter by tag | `i-rs-remind list --tag work` | Filtered | ✅ PASS |

### Get Tests

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| Get reminder | `i-rs-remind get test` | Shown | ✅ PASS |
| Get missing | `i-rs-remind get nonexistent` | Error | ✅ PASS |

### Done Tests

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| Mark done | `i-rs-remind done test` | Marked complete | ✅ PASS |
| Done missing | `i-rs-remind done nonexistent` | Error | ✅ PASS |

### Update Tests

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| Update date | `i-rs-remind update test --event-date 2026-01-01` | Updated | ✅ PASS |
| Update time | `i-rs-remind update test --event-date 2025-12-31 15:00` | Time updated | ✅ PASS |
| Update title | `i-rs-remind update test --title "New"` | Updated | ✅ PASS |

### Delete Tests

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| Delete | `i-rs-remind delete test` | Deleted | ✅ PASS |

## Summary

- **Total Tests**: 14
- **Passed**: 14
- **Success Rate**: 100%