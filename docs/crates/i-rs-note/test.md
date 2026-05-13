# i-rs-note Test Records

## Test Environment

- **Platform**: macOS, Linux, Windows
- **Test Date**: 2024-01-15

## Test Cases

### Add Tests

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| Add simple note | `i-rs-note add test --content "hello"` | Added | ✅ PASS |
| Add with title | `i-rs-note add test --title "Title" --content "content"` | Added | ✅ PASS |
| Add with tags | `i-rs-note add test --tag work --tag dev --content "x"` | Tags stored | ✅ PASS |
| Add duplicate | `i-rs-note add test --content "x"` | Error | ✅ PASS |

### List Tests

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| List all | `i-rs-note list` | All shown | ✅ PASS |
| Filter by tag | `i-rs-note list --tag work` | Filtered | ✅ PASS |

### Get Tests

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| Get note | `i-rs-note get test` | Shown | ✅ PASS |
| Get missing | `i-rs-note get nonexistent` | Error | ✅ PASS |

### Update/Delete Tests

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| Update content | `i-rs-note update test --content "new"` | Updated | ✅ PASS |
| Update tags | `i-rs-note update test --tag new` | Tags replaced | ✅ PASS |
| Delete | `i-rs-note delete test` | Deleted | ✅ PASS |

## Summary

- **Total Tests**: 11
- **Passed**: 11
- **Success Rate**: 100%