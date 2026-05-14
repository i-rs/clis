# AGENTS.md - i-rs CLI Tools

## Project Overview

Rust monorepo with 38+ cross-platform CLI tools for personal data management, plus 1 shared core library.

## Project Structure

```
i-rs-clis/
├── crates/
│   ├── i-rs-core/          # Shared core library
│   ├── i-rs-server/        # Server management
│   ├── i-rs-password/      # Password management
│   ├── i-rs-bookmark/      # Bookmark management
│   ├── i-rs-note/          # Note management
│   ├── i-rs-domain/        # Domain expiry tracking
│   ├── i-rs-remind/        # Event reminders
│   ├── i-rs-weight/        # Weight tracking
│   ├── i-rs-mood/          # Mood tracking
│   ├── i-rs-todo/          # Todo tracking
│   └── ...                 # 30+ more CLI tools
├── docs/                   # VitePress documentation
│   └── .vitepress/
│       └── config.ts       # Documentation sidebar config
├── skills/                 # AI skill documents
├── Cargo.toml              # Workspace config
├── README.md
├── SPEC.md                 # Detailed specifications
└── AGENTS.md               # This file
```

## CLI Tools Summary

| Tool | Description | Special Commands |
|------|-------------|-----------------|
| i-rs-server | Server management | suggest |
| i-rs-password | Password management | - |
| i-rs-bookmark | Bookmark management | - |
| i-rs-note | Note management | - |
| i-rs-domain | Domain expiry tracking | - |
| i-rs-remind | Event reminders | done |
| i-rs-weight | Weight tracking | chart, stats |
| i-rs-mood | Mood tracking | calendar |
| i-rs-todo | Todo tracking | done |
| i-rs-water | Water intake tracking | - |
| i-rs-step | Step counting | - |
| i-rs-dose | Medicine dosage | - |
| i-rs-cycle | Menstrual cycle | - |
| i-rs-sit | Sedentary reminder | - |
| i-rs-allergy | Allergy tracking | - |
| i-rs-cal | Calorie estimation | - |
| i-rs-fast | Fasting tracking | - |
| i-rs-sub | Subscription tracking | - |
| i-rs-bestby | Best-by date tracking | - |
| i-rs-ledger | Accounting | - |
| i-rs-recur | Recurring expenses | - |
| i-rs-kv | Key-value storage | - |
| i-rs-keys | API key management | - |
| i-rs-meal | Meal tracking | - |
| i-rs-pig | Craving tracking | - |
| i-rs-tick | Duration tracking | - |
| i-rs-spark | Inspiration capture | - |
| i-rs-want | Wish list | - |
| i-rs-sheet | Bedsheet replacement | - |
| i-rs-toothbrush | Toothbrush replacement | - |
| i-rs-towel | Towel replacement | - |
| i-rs-bed | Mattress/pillow replacement | - |
| i-rs-ac | AC cleaning | - |
| i-rs-filter | Filter cleaning | - |
| i-rs-purify | Water purifier filter | - |
| i-rs-feedpet | Pet feeding | - |
| i-rs-petbath | Pet bathing | - |
| i-rs-walkdog | Dog walking | - |
| i-rs-aqua | Aquarium maintenance | - |

## Build & Development

```bash
# Build all crates
cargo build

# Build specific tool
cargo build -p i-rs-mood

# Run specific tool
cargo run -p i-rs-mood -- --help

# Test specific crate
cargo test -p i-rs-mood

# Check for warnings
cargo check
```

## i-rs-core Shared Library

The `i-rs-core` crate provides shared functionality for all CLI tools:

```
crates/i-rs-core/src/
├── lib.rs                    # Public API exports
├── storage/                  # Generic Storage<T> for JSON persistence
├── presentation/             # Output formatting
│   ├── mod.rs              # print_error/success/header/warning + OutputFormat
│   └── output.rs           # JSON output formatting
└── utils/
    ├── date.rs             # parse_date()
    └── validation.rs       # validate_name/validate_url/validate_weight
```

### i-rs-core Exports

```rust
// Storage
pub use i_rs_core::storage::{Storage, filter_by_tag, HasTags};

// Presentation
pub use i_rs_core::presentation::{print_error, print_header, print_success, print_warning, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item, output_error};

// Utils
pub use i_rs_core::utils::parse_date;
pub use i_rs_core::utils::validation::{validate_name, validate_url, validate_weight, ValidationError};
```

## Crate Structure

Each CLI crate follows this pattern:
```
crates/i-rs-{name}/
├── src/
│   ├── main.rs           # CLI entry point (clap) + --json global flag
│   ├── commands/         # add, delete, get, list, update, example, skill, [special]
│   ├── models/           # Data structs with serde + tabled
│   ├── storage/          # keyring + JSON file (uses i-rs-core Storage)
│   └── presentation/     # tabled output, colors, charts
├── Cargo.toml
└── README.md
```

## New Crate Workflow (CHECKLIST)

When creating a new crate `i-rs-{name}`, follow this **complete checklist**:

### Step 1: Create Directory Structure
```bash
mkdir -p crates/i-rs-{name}/src/{models,storage,commands,presentation}
mkdir -p docs/crates/i-rs-{name}
mkdir -p skills/i-rs-{name}
```

### Step 2: Create Cargo.toml
```toml
[package]
name = "i-rs-{name}"
version.workspace = true
edition.workspace = true

[dependencies]
i-rs-core = { path = "../i-rs-core" }
clap.workspace = true
anyhow.workspace = true
serde.workspace = true
serde_json.workspace = true
dirs.workspace = true
keyring.workspace = true     # if storing passwords/keys
keyring-core.workspace = true
tabled.workspace = true
owo-colors.workspace = true
chrono.workspace = true
uuid.workspace = true        # if using UUIDs
```

### Step 3: Create Source Files
- `src/models/mod.rs` - Entity struct + Row struct (Tabled) + ListItem
- `src/storage/mod.rs` - JSON persistence (uses i-rs-core Storage) + keyring if needed
- `src/presentation/mod.rs` - Table formatting + count printing
- `src/commands/mod.rs` - Command module exports
- `src/commands/add.rs` - Add command
- `src/commands/delete.rs` - Delete command
- `src/commands/get.rs` - Get command
- `src/commands/list.rs` - List command
- `src/commands/update.rs` - Update command (if applicable)
- `src/commands/example.rs` - Example command
- `src/commands/skill.rs` - Skill command
- `src/main.rs` - CLI parsing with --json global flag

### Step 4: Create README.md (REQUIRED!)
```markdown
# i-rs-{name}

[Description] CLI tool for [purpose].

## Features

- Feature 1
- Feature 2
- Tag support

## Install

```bash
npm install -g @i-rs/i-rs-{name}
# or
brew install i-rs/homebrew-tap/i-rs-{name}
```

## Quick Start

```bash
# [basic usage]
i-rs-{name} add ...

# List all
i-rs-{name} list
```

## Data Storage

- macOS: `~/.config/i-rs/{name}.json`
- Linux: `~/.config/i-rs/{name}.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0
```

### Step 5: Create Docs (REQUIRED!)
Create 4 files in `docs/crates/i-rs-{name}/`:

**index.md** - Overview and quick start
**usage.md** - Detailed command reference
**examples.md** - Extensive usage examples
**test.md** - Test records for verification

### Step 6: Create Skills (REQUIRED!)
Create `skills/i-rs-{name}/SKILL.md`:
```markdown
---
name: "i-rs-{name}"
description: "[One-line description]. Invoke when [use cases]."
---

# i-rs-{name}

[Description] CLI tool.

## Storage

- Config: `~/.config/i-rs/{name}.json`

## Commands

### add
...

### list
...

### get
...

### delete
...

## Examples

```bash
i-rs-{name} add ...
```
```

### Step 7: Update Workspace Cargo.toml
Add `"crates/i-rs-{name}"` to the `members` array in root `Cargo.toml`.

### Step 8: Update VitePress Config (REQUIRED!)
Add sidebar entry in `docs/.vitepress/config.ts`:
```typescript
{
  text: 'i-rs-{name}',
  collapsed: true,
  items: [
    { text: 'Overview', link: '/crates/i-rs-{name}/' },
    { text: 'Usage', link: '/crates/i-rs-{name}/usage' },
    { text: 'Examples', link: '/crates/i-rs-{name}/examples' },
    { text: 'Test', link: '/crates/i-rs-{name}/test' }
  ]
}
```

### Step 9: Build and Verify
```bash
cargo build -p i-rs-{name}
cargo check
```

---

## ⚠️ IMPORTANT: Incomplete Crate Checklist

If you encounter a crate that is missing documentation, verify and complete:

- [ ] `crates/i-rs-{name}/README.md` exists
- [ ] `docs/crates/i-rs-{name}/index.md` exists
- [ ] `docs/crates/i-rs-{name}/usage.md` exists
- [ ] `docs/crates/i-rs-{name}/examples.md` exists
- [ ] `docs/crates/i-rs-{name}/test.md` exists
- [ ] `skills/i-rs-{name}/SKILL.md` exists
- [ ] `docs/.vitepress/config.ts` has sidebar entry for this crate
- [ ] `Cargo.toml` workspace has this crate in members

## Key Conventions

- **Workspace deps**: All dependencies defined in root `Cargo.toml`, crates use `.workspace = true`
- **i-rs-core dependency**: All crates depend on `i-rs-core = { path = "../i-rs-core" }`
- **Passwords**: Always store in OS keychain (keyring crate), NEVER in JSON config
- **Data location**: `~/.config/i-rs/` (override with `CONFIG_DIR` env var)
- **Date handling**: chrono with `ts_seconds` serde format
- **Error handling**: `anyhow::Result<()>` with `?` operator
- **CLI framework**: clap with derive macro, snake_case params auto-convert to kebab-case
- **Output**: tabled with cyan headers, green rows
- **JSON output**: All commands support `--json` flag for JSON output

## Common Patterns

### Model Struct
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
```

### Sensitive Fields
```rust
#[serde(skip)]
#[allow(dead_code)]
pub password: Option<String>,
```

### Time Calculation (for domain/remind/weight/mood)
```rust
pub fn days_until(&self) -> i64 {
    (self.event_date - Utc::now()).num_days()
}
```

### presentation/mod.rs Pattern
```rust
use crate::models::{Entity, EntityRow};
use owo_colors::OwoColorize;
use tabled::{settings::Color, settings::object::Rows, settings::object::Segment, settings::style::BorderColor, settings::style::Style, settings::themes::Colorization, Table};

pub use i_rs_core::presentation::{print_error, print_header, print_success, print_warning, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item, output_error};

pub fn format_table(entities: &[&Entity]) -> String {
    let rows: Vec<EntityRow> = entities
        .iter()
        .map(|e| EntityRow::from_entity(e))
        .collect();

    Table::new(&rows)
        .with(Style::modern_rounded())
        .modify(Segment::all(), BorderColor::filled(Color::FG_CYAN))
        .with(Colorization::exact([Color::FG_CYAN | Color::BOLD], Rows::first()))
        .with(Colorization::exact([Color::FG_GREEN], Rows::new(1..)))
        .to_string()
}

pub fn print_entity_count(count: usize) {
    println!("\n{} {} entities", "Total:".dimmed(), count.to_string().cyan());
}
```

## Global Commands (All Crates)

### example Command
Show usage examples for AI/human to quickly understand the CLI.
```bash
i-rs-{name} example
```

### skill Command
View AI skill documentation integrated into CLI itself.
```bash
i-rs-{name} skill          # Show raw skill document
i-rs-{name} skill summary  # Show summary
i-rs-{name} skill content  # Show content
```

## JSON Output

All commands support `--json` global flag for JSON output:

```bash
i-rs-{name} list --json
i-rs-{name} get <name> --json
```

### JSON Response Format

**List Response:**
```json
{
  "success": true,
  "data": [...],
  "meta": {
    "count": 10,
    "filter": "work"
  }
}
```

**Item Response:**
```json
{
  "success": true,
  "data": {...}
}
```

**Error Response:**
```json
{
  "success": false,
  "error": {
    "code": "NOT_FOUND",
    "message": "Entry 'xxx' not found"
  }
}
```

## Input Validation

Use `i-rs-core` validation functions in `add` and `update` commands:

```rust
use i_rs_core::{validate_name, validate_url, validate_weight, ValidationError};

// In add/update command
if let Err(e) = validate_name(&name) {
    print_error(&e.message);
    anyhow::bail!("{}", e.message);
}
```

### Validation Rules

| Function | Rules |
|----------|-------|
| `validate_name` | Non-empty, ≤100 chars, no `/ \ : * ? " < > \|` |
| `validate_url` | Non-empty, starts with `http://` or `https://`, ≤2000 chars |
| `validate_weight` | > 0, ≤1000 kg |

## Bug Prevention

### Store Loading Pattern (CORRECT)
```rust
// CORRECT: Load once, use mutable reference
let mut store = storage::load_store()?;

if store.entries.contains_key(&name) {
    print_error(&format!("Entry '{}' already exists", name));
    anyhow::bail!("Entry '{}' already exists", name);
}

// ... create entity ...

storage::add_entry(&mut store, entry);
storage::save_store(&store)?;
```

### Store Loading Pattern (INCORRECT - BUG!)
```rust
// WRONG: Double loading - wastes I/O and causes bugs
let store = storage::load_store()?;
if store.entries.contains_key(&name) { ... }

// ... later ...

let mut store = storage::load_store()?;  // BUG: Reloading!
storage::add_entry(&mut store, entry);
```

## Release Process

```bash
# Update version in root Cargo.toml (workspace.package.version)
git tag v0.0.x
git push origin v0.0.x
```

CI (cargo-dist) auto-builds and publishes to:
- GitHub Releases
- npm (`@i-rs/i-rs-*`)
- Homebrew (`i-rs/homebrew-tap/i-rs-*`)

## Important Files

- `SPEC.md` - Detailed project specification (Chinese)
- `AGENTS.md` - This file (development workflow for AI)
- `Cargo.toml` - Workspace config
- `docs/.vitepress/config.ts` - Documentation sidebar config
- `.github/workflows/release.yml` - CI/release automation

## VitePress Documentation

Documentation at `docs/` uses VitePress. Each crate has 4 pages in `docs/crates/i-rs-{name}/`:
- `index.md` - Overview
- `usage.md` - Command reference
- `examples.md` - Detailed examples
- `test.md` - Test records