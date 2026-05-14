# i-rs-core Test Records

## Unit Tests

i-rs-core is a library crate. Run tests with:

```bash
cargo test -p i-rs-core
```

## Manual Testing

### Storage

```bash
# Create a test directory
CONFIG_DIR=/tmp/i-rs-test

# Run a CLI tool that uses Storage
CONFIG_DIR=/tmp/i-rs-test cargo run -p i-rs-todo -- add test-1 --title "Test"
# Verify file created
ls /tmp/i-rs-test/todo.json
```

### Date Parsing

```rust
use i_rs_core::parse_date;

// Valid formats
assert!(parse_date("2024-01-15").is_ok());
assert!(parse_date("2024/01/15").is_ok());
assert!(parse_date("15-01-2024").is_ok());
assert!(parse_date("invalid").is_err());
```

### Validation

```rust
use i_rs_core::validate_name;

// Valid
assert!(validate_name("hello").is_ok());
assert!(validate_name("a".repeat(100)).is_ok());

// Invalid
assert!(validate_name("").is_err());
assert!(validate_name("hello/world").is_err());
assert!(validate_name("a".repeat(101)).is_ok());
```

## Test Coverage

- Storage load/save round-trip
- Date parsing (multiple formats)
- URL validation
- Name validation (empty, length, special chars)
- Weight validation
- Amount validation
- JSON output formatting
- Theme loading
- Custom config directory
