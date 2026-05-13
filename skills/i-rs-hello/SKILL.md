---
name: "i-rs-hello"
description: "Simple hello world CLI. Invoke when user wants to test CLI functionality or needs a basic greeting tool."
---

# i-rs-hello

Simple hello world CLI tool.

## Usage

```bash
i-rs-hello [OPTIONS]
```

## Options

| Option | Short | Description |
| -------- | ------- | ------------- |
| `--name <NAME>` | `-n` | Name to greet (default: "World") |

## Examples

```bash
# Default greeting
i-rs-hello
# Output: Hello, World!

# With name
i-rs-hello --name Alice
# Output: Hello, Alice!

# Short flag
i-rs-hello -n Bob
# Output: Hello, Bob!
```
