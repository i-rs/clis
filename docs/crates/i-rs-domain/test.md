# i-rs-domain Test Records

## Test Environment

- **Platform**: macOS, Linux, Windows
- **Test Date**: 2024-01-15

## Test Cases

### Add Tests

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| Add domain | `i-rs-domain add example.com 2025-12-31` | Added | ✅ PASS |
| Add with registrar | `i-rs-domain add example.com 2025-12-31 --registrar GoDaddy` | Registrar stored | ✅ PASS |
| Add with password | `i-rs-domain add example.com 2025-12-31 --password xxx` | Password in keychain | ✅ PASS |
| Add with tags | `i-rs-domain add example.com 2025-12-31 --tag important` | Tags stored | ✅ PASS |
| Add duplicate | `i-rs-domain add example.com 2025-12-31` | Error | ✅ PASS |

### List Tests

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| List all | `i-rs-domain list` | All shown | ✅ PASS |
| Filter by tag | `i-rs-domain list --tag important` | Filtered | ✅ PASS |

### Get Tests

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| Get domain | `i-rs-domain get example.com` | Shown | ✅ PASS |
| Show password | `i-rs-domain get example.com --show-password` | Password visible | ✅ PASS |

### Update Tests

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| Update expiry | `i-rs-domain update example.com --expiry-date 2026-12-31` | Updated | ✅ PASS |
| Update registrar | `i-rs-domain update example.com --registrar Cloudflare` | Updated | ✅ PASS |
| Update password | `i-rs-domain update example.com --password new` | Keychain updated | ✅ PASS |

### Delete Tests

| Test Case | Command | Expected | Status |
|-----------|---------|----------|--------|
| Delete domain | `i-rs-domain delete example.com` | Deleted + keychain | ✅ PASS |

## Summary

- **Total Tests**: 12
- **Passed**: 12
- **Success Rate**: 100%