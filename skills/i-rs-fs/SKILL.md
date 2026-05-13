---
name: "i-rs-fs"
description: "File system operations CLI (read/write/count lines). Invoke when user needs to perform file operations via command line."
---

# i-rs-fs

File system operations CLI tool.

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
# Read file
i-rs-fs read README.md

# Write file
i-rs-fs write output.txt "Hello, World!"

# Count lines
i-rs-fs lines myfile.txt
```
