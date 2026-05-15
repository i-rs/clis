# i-rs — Unified CLI for all i-rs tools

Run any i-rs tool as a subcommand. Each tool can also be used independently.

## Install

```bash
# Via npm
npm install -g @i-rs/i-rs

# Via Homebrew
brew install i-rs/homebrew-tap/i-rs

# Or build from source
git clone https://github.com/i-rs/clis.git
cd clis
cargo build -p i-rs
```

> **Note:** This package only installs the dispatcher. Individual tools must be installed separately:
> ```bash
> npm install -g @i-rs/i-rs-todo
> # or
> cargo install -p i-rs-todo
> ```

## Usage

```bash
# Run any tool as a subcommand
i-rs todo list
i-rs mood add today happy
i-rs weight add 75

# Forward flags to the tool
i-rs todo --help
i-rs todo list --pending

# Tools also work independently
i-rs-todo add work --priority high
i-rs-mood list --days 7 --calendar
```

## Available Tools

Run `i-rs --help` to list all available tools, or use any of them:

```bash
i-rs todo          # Todo tracking
i-rs mood         # Mood tracking
i-rs weight       # Weight tracking
# ... 70+ tools
```

## How It Works

`i-rs` is a lightweight dispatcher. When you run `i-rs todo list`, it looks for `i-rs-todo` in your PATH and forwards the arguments. This is the same pattern used by `git`, `cargo`, `docker`, and `kubectl` for plugin systems.

## License

MIT OR Apache-2.0
