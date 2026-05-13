# i-rs-bookmark Test Records

## Test Environment

- **Platform**: macOS, Linux, Windows
- **Test Date**: 2024-01-15

## Test Cases

### Add Tests

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| Add bookmark | `i-rs-bookmark add github https://github.com` | Added | ✅ PASS |
| Add with credentials | `i-rs-bookmark add aws https://aws.amazon.com --account admin --password pass` | Added + keychain | ✅ PASS |
| Add duplicate | `i-rs-bookmark add github https://github.com` | Error | ✅ PASS |

### List Tests

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| List all | `i-rs-bookmark list` | All shown | ✅ PASS |
| Filter by tag | `i-rs-bookmark list --tag work` | Filtered | ✅ PASS |

### Get Tests

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| Get details | `i-rs-bookmark get github` | Shown | ✅ PASS |
| Show password | `i-rs-bookmark get aws --show-password` | Password visible | ✅ PASS |

### Update/Delete Tests

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| Update URL | `i-rs-bookmark update github --url https://github.com/enterprise` | Updated | ✅ PASS |
| Delete | `i-rs-bookmark delete github` | Deleted + keychain | ✅ PASS |

## Summary

- **Total Tests**: 12
- **Passed**: 12
- **Success Rate**: 100%