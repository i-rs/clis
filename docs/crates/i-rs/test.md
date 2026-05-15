# Test

## Manual Testing

### Tool Dispatch

```bash
# Test tool discovery
i-rs
# Expected: List of all available tools

# Test specific tool
i-rs todo list
# Expected: Dispatches to i-rs-todo list

# Test with arguments
i-rs todo add "Test task" --priority high
# Expected: Task added via i-rs-todo
```

### Error Cases

```bash
# Non-existent tool
i-rs nonexistent
# Expected: Error message "'i-rs-nonexistent' is not installed"

# Invalid flag
i-rs --invalid
# Expected: Error message "unknown flag '--invalid'"
```

### Version

```bash
i-rs --version
# Expected: "i-rs 0.0.2"
```

### Help

```bash
i-rs --help
# Expected: Usage information with tool list
```

## Build

```bash
cargo build -p i-rs
```

## CI

The umbrella crate is built and published as part of the workspace CI pipeline.
