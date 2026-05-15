---
name: "i-rs"
description: "Unified CLI dispatcher. Run any i-rs tracking tool via `i-rs <TOOL> [ARGS]...`."
---

# i-rs — Unified CLI Dispatcher

`i-rs` is a lightweight dispatcher that forwards commands to individual `i-rs-*` tools. It follows the same plugin pattern as `git`, `cargo`, and `docker`.

## Usage

```bash
i-rs <TOOL> [ARGS]...
```

Dispatches to `i-rs-<TOOL>` with the given arguments.

## Commands

| Command | Description |
|---------|-------------|
| `i-rs` | List all available tools |
| `i-rs --help` | Show usage |
| `i-rs --version` | Show version |
| `i-rs <TOOL>` | Dispatch to tool |

## Examples

```bash
i-rs todo list
i-rs mood add today happy
i-rs weight add 75
i-rs todo --help
```

## Note

Each tool must be installed separately (`i-rs-<TOOL>` in PATH). This package only installs the dispatcher.
