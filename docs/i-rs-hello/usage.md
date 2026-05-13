# i-rs-hello Usage Guide

## Install

```bash
npm install -g @i-rs/hello
# or
brew install i-rs/hello-tap/hello
```

## Basic Usage

```bash
# Default greeting
i-rs-hello
# Output: Hello, World!

# With name parameter
i-rs-hello --name Alice
# Output: Hello, Alice!

# Short flag
i-rs-hello -n Bob
# Output: Hello, Bob!
```

## Options

| Option | Short | Description |
| -------- | ------- | ------------- |
| `--name <NAME>` | `-n` | Name to greet |
| `--help` | `-h` | Show help |
| `--version` | `-V` | Show version |
