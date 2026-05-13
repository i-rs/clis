# i-rs-fs Usage Guide

## Install

```bash
npm install -g @i-rs/fs
# or
brew install i-rs/fs-tap/fs
```

## Commands

### read

Read and display file content.

```bash
i-rs-fs read <FILE>
```

### write

Write content to a file.

```bash
i-rs-fs write <FILE> <CONTENT>
```

### lines

Count lines in a file.

```bash
i-rs-fs lines <FILE>
```

## Examples

```bash
# Read a file
i-rs-fs read README.md

# Write content
i-rs-fs write output.txt "Hello, World!"

# Count lines
i-rs-fs lines myfile.txt
```
