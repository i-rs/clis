# i-rs-password Test Records

## Test Environment

- **Platform**: macOS 14.0 (Sonoma), Linux (Ubuntu 22.04), Windows 11
- **Rust Version**: 1.75+
- **Test Date**: 2024-01-15

## Manual Test Cases

### Add Command Tests

| Test Case | Command | Expected Result | Status |
|-----------|---------|-----------------|--------|
| Add entry with URL | `i-rs-password add github https://github.com` | Entry added | ✅ PASS |
| Add entry with account | `i-rs-password add github https://github.com --account user` | Account stored | ✅ PASS |
| Add entry with password | `i-rs-password add github https://github.com --password secret` | Password in keychain | ✅ PASS |
| Add entry with tags | `i-rs-password add github https://github.com --tag work --tag dev` | Tags stored | ✅ PASS |
| Add entry with remarks | `i-rs-password add github https://github.com --remark "test"` | Remarks stored | ✅ PASS |
| Add duplicate name | `i-rs-password add github https://github.com` (twice) | Error: Duplicate | ✅ PASS |
| Add entry without URL | `i-rs-password add github` | Error: Missing URL | ✅ PASS |

### List Command Tests

| Test Case | Command | Expected Result | Status |
|-----------|---------|-----------------|--------|
| List all entries | `i-rs-password list` | All entries displayed | ✅ PASS |
| List with tag filter | `i-rs-password list --tag work` | Filtered results | ✅ PASS |
| List with non-existent tag | `i-rs-password list --tag nonexistent` | Empty list | ✅ PASS |

### Get Command Tests

| Test Case | Command | Expected Result | Status |
|-----------|---------|-----------------|--------|
| Get existing entry | `i-rs-password get github` | Details displayed | ✅ PASS |
| Get with hidden password | `i-rs-password get github` | Password hidden | ✅ PASS |
| Get with visible password | `i-rs-password get github --show-password` | Password shown | ✅ PASS |
| Get non-existent entry | `i-rs-password get nonexistent` | Error: Not found | ✅ PASS |

### Update Command Tests

| Test Case | Command | Expected Result | Status |
|-----------|---------|-----------------|--------|
| Update URL | `i-rs-password update github --url https://github.com` | URL updated | ✅ PASS |
| Update account | `i-rs-password update github --account newuser` | Account updated | ✅ PASS |
| Update password | `i-rs-password update github --password newpass` | Password in keychain | ✅ PASS |
| Update tags | `i-rs-password update github --tag newtag` | Tags replaced | ✅ PASS |
| Update remarks | `i-rs-password update github --remark "new"` | Remarks replaced | ✅ PASS |
| Update non-existent | `i-rs-password update nonexistent --url http://x.com` | Error: Not found | ✅ PASS |

### Delete Command Tests

| Test Case | Command | Expected Result | Status |
|-----------|---------|-----------------|--------|
| Delete existing entry | `i-rs-password delete github` | Entry and keychain deleted | ✅ PASS |
| Delete non-existent | `i-rs-password delete nonexistent` | Error: Not found | ✅ PASS |

## Security Tests

| Test Case | Description | Expected Result | Status |
|-----------|-------------|-----------------|--------|
| Password in keychain only | Add with password | Not in JSON | ✅ PASS |
| Keychain retrieval | Get with --show-password | Returns keychain value | ✅ PASS |
| Keychain deletion | Delete entry | Keychain entry removed | ✅ PASS |

## Platform Tests

### macOS

| Test | Description | Status |
|------|-------------|--------|
| Keychain storage | Password in Keychain | ✅ PASS |
| Keychain retrieval | Retrieve password | ✅ PASS |
| Keychain deletion | Delete on entry delete | ✅ PASS |

### Linux

| Test | Description | Status |
|------|-------------|--------|
| Secret Service | Works with libsecret | ✅ PASS |
| Keyutils fallback | Works with keyutils | ✅ PASS |

### Windows

| Test | Description | Status |
|------|-------------|--------|
| Credential Manager | Stores in Credential Manager | ✅ PASS |
| Retrieval | Retrieves correctly | ✅ PASS |

## Summary

- **Total Tests**: 22
- **Passed**: 22
- **Failed**: 0
- **Success Rate**: 100%

## Test Coverage

- ✅ Add command
- ✅ List command with filters
- ✅ Get command with password visibility
- ✅ Update command
- ✅ Delete command
- ✅ Keychain integration
- ✅ Cross-platform support