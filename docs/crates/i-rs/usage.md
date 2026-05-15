# Usage

## Basic Usage

```bash
i-rs <TOOL> [ARGS]...
```

The dispatcher forwards all arguments to the corresponding `i-rs-<TOOL>` binary.

## Commands

| Command | Description |
|---------|-------------|
| `i-rs` | List all available tools |
| `i-rs --help` | Show usage information |
| `i-rs --version` | Show version |
| `i-rs <TOOL>` | Run a specific tool |
| `i-rs <TOOL> --help` | Show tool-specific help |

## Tool Dispatch

When you run `i-rs todo add "Buy groceries" --priority high`:

1. `i-rs` receives `todo add "Buy groceries" --priority high`
2. It looks for `i-rs-todo` in your PATH
3. It executes `i-rs-todo add "Buy groceries" --priority high`
4. All output from the tool is passed through directly

## Listing Tools

Run `i-rs` with no arguments to list all available tools:

```bash
$ i-rs
  server     Server management
  password   Password management
  todo       Todo tracking
  mood       Mood tracking
  ... (70+ tools)
```

## Help

```bash
# List available tools
i-rs

# Show usage
i-rs --help

# Show version
i-rs --version

# Get help for a specific tool
i-rs todo --help
```

## Installation

```bash
# Via npm
npm install -g @i-rs/i-rs

# Via Homebrew
brew install i-rs/homebrew-tap/i-rs

# Or build from source
cargo build -p i-rs
```

Individual tools must be installed separately:

```bash
npm install -g @i-rs/i-rs-todo
# or
cargo install -p i-rs-todo
```
