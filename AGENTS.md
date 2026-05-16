# AGENTS.md - i-rs CLI Tools

## Project Overview

Rust monorepo with **70 cross-platform CLI tools** for personal data management, plus special crates, MCP server, and browser extensions:

- **`i-rs`** - Meta CLI (unified entry point)
- **`i-rs-core`** - Shared core library
- **`i-rs-api`** - REST API server
- **`i-rs-mcp`** - MCP (Model Context Protocol) server
- **`extensions/`** - Browser extensions via Native Messaging

**Current state:** `cargo check` — 0 errors, 0 warnings. 21 unit tests in i-rs-core, 32 integration tests in i-rs-api.

## Project Structure

```
i-rs-clis/
├── crates/
│   ├── i-rs/               # Meta CLI (unified entry point for all tools)
│   ├── i-rs-core/          # Shared core library (macros, Storage, presentation, utils)
│   ├── i-rs-api/           # REST API server (Axum)
│   ├── i-rs-mcp/           # MCP server (JSON-RPC 2.0 over stdio)
│   ├── i-rs-{name}...      # 70 CLI tools
├── extensions/              # Browser extensions (Native Messaging)
│   ├── i-rs-kv-chrome/     # Chrome extension for i-rs-kv
│   ├── i-rs-native-msg/    # Native messaging host (Rust)
│   └── install-*.sh        # Installation scripts
├── docs/                   # VitePress documentation
│   └── .vitepress/
│       └── config.ts       # Documentation sidebar config
├── skills/                 # AI skill documents (70 crates)
├── .github/workflows/
│   ├── release.yml         # cargo-dist auto-publish
│   └── check.yml           # CI: check + clippy + fmt
├── deny.toml               # cargo-deny config
├── rust-toolchain.toml     # Pinned Rust toolchain
├── Cargo.lock              # Committed for reproducible builds
├── Cargo.toml              # Workspace config
├── README.md
├── SPEC.md                 # Detailed specifications (Chinese)
└── AGENTS.md               # This file
```

## Special Crates

These crates are **not standard CLI tools** and have different development requirements:

| Crate | Type | Description |
|-------|------|-------------|
| `i-rs` | Meta CLI | Unified entry point that delegates to all 70 CLI tools |
| `i-rs-core` | Shared Library | Core library providing `Storage<T>`, macros, presentation, and utilities |
| `i-rs-api` | REST API | REST API server built with Axum, wrapping CLI tools as HTTP endpoints |
| `i-rs-mcp` | MCP Server | MCP (Model Context Protocol) server exposing CLI tools as JSON-RPC 2.0 tools over stdio |

### i-rs (Meta CLI)

`i-rs` is a **meta wrapper** that provides a unified CLI entry point:

```bash
i-rs weight list     # Calls i-rs-weight list
i-rs mood calendar   # Calls i-rs-mood calendar
```

It does NOT have its own models/storage/commands - it only routes to other tools.

### i-rs-core (Shared Library)

`i-rs-core` is the **foundation** for all 70 CLI tools:

- **Must NOT depend on any other i-rs-* crates**
- Provides: `Storage<T>`, `create_store!`, `skill_command!`, `exit_on_error!`, presentation helpers, validation utilities
- 21 unit tests verify core functionality

### i-rs-api (REST API)

`i-rs-api` is a **REST API** built with Axum:

- **`make_app_tools!` macro** — generates `AppState` struct, `load_state()`, and `build_base_router()` from a single list of 70 (field, store_type, filename) tuples
- **`build.rs`** — auto-generates `routes.rs` module declarations from `src/routes/*.rs` files
- **70 CRUD route modules** — one per CLI tool, each with `GET /` (list), `POST /` (create), `GET /{id}` (get), `DELETE /{id}` (delete), `PATCH /{id}` (update)
- **Service layer reuse** — API endpoints call `i_rs_{name}::service::*` directly, NOT CLI `commands::handle_*` functions
- **Per-tool data endpoints** — `GET /data/export`, `POST /data/import`, `DELETE /data/clear`
- **`update.rs`** — generic `merge_entry()` for partial JSON updates: shallow merge for primitives/arrays, deep merge for nested objects, null field removal, and auto-`updated_at` timestamp
- **`response.rs`** — unified `ApiError` / `ApiResult` with consistent JSON error responses
- **All update endpoints use `PATCH`** (not `PUT`) for semantic partial updates
- **32 integration tests** — covering health, CRUD, PATCH, 404, bad request, data export/clear across all 3 store patterns (String-keyed BTreeMap, NaiveDate-keyed BTreeMap, Vec-based)
- Uses `i-rs-core` but NOT the standard CLI crate pattern
- Does NOT have: `storage/mod.rs`, `commands/`, `models/`, `presentation/`

### i-rs-mcp (MCP Server)

`i-rs-mcp` is an **MCP (Model Context Protocol) server** that exposes all 70 i-rs CLI tools as MCP tools:

- **JSON-RPC 2.0 over stdio** — communicates via stdin/stdout using the MCP wire protocol
- **`make_mcp_tools!` macro** — generates `AppState`, `load_state()`, `get_tool_definitions()`, and `handle_tool_call()` from a single list of 65 (field, store_type, entity_type, map_field, key_type) tuples
- **Each tool = 4 operations** — list, get, add, delete (with JSON-RPC `tools/call` dispatch)
- **`SharedStore<T>`** — thread-safe `Arc<RwLock<T>>` wrapper with auto-flush on write
- **Runtime JSON reflection** — `store_values()` and `store_get()` use serde_json runtime serialization to handle different BTreeMap field names and key types
- **`remove_by_key!` macro** — three arms for key-type-specific remove logic (s=String, d=NaiveDate, u=Uuid)
- **`fill_defaults()`** — injects `id`/`created_at`/`updated_at` before entity deserialization
- **65 BTreeMap-based tools registered** (Phase 1); 5 non-standard stores (car/plant/project/budget/goal) deferred to Phase 2
- **Does NOT follow standard CLI crate pattern** — no `main.rs` clap CLI, no `commands/`, `models/`, `storage/`, `presentation/` directories

```
crates/i-rs-mcp/
├── Cargo.toml              # Dependencies on all 68 i-rs-* crates + serde/serde_json/uuid/chrono/paste
└── src/
    ├── main.rs             # MCP server entry: macros + session loop
    └── transport.rs        # JSON-RPC 2.0 stdio transport layer
```

## Browser Extensions (Native Messaging)

Browser extensions can communicate with i-rs tools using Chrome/Firefox Native Messaging.

### Architecture

```
┌─────────────────┐     Native Messaging      ┌──────────────────┐
│  Browser        │ ◄──── JSON over stdio ────► │  Native Host     │
│  Extension JS   │                            │  (Rust binary)   │
└─────────────────┘                            └────────┬─────────┘
                                                        │
                                                        │ reads/writes
                                                        ▼
                                               ┌──────────────────┐
                                               │  ~/.config/i-rs/ │
                                               │      kv.json     │
                                               └──────────────────┘
```

### Directory Structure

```
extensions/
├── i-rs-kv-chrome/          # Chrome extension for i-rs-kv
│   ├── manifest.json        # Extension manifest (Manifest V3)
│   ├── popup.html/css/js    # Popup UI
│   ├── background.js        # Service worker
│   └── icons/               # Extension icons
├── i-rs-native-msg/         # Native Messaging host (Rust)
│   ├── src/main.rs          # Host implementation
│   └── Cargo.toml           # Dependencies
├── install-macos.sh         # One-click installer for macOS
└── install-linux.sh         # One-click installer for Linux
```

### Message Protocol

Messages are JSON objects with an `action` field:

```json
{ "action": "List" }
{ "action": "Get", "key": "my-key" }
{ "action": "Set", "key": "my-key", "value": "my-value" }
{ "action": "Delete", "key": "my-key" }
{ "action": "Search", "query": "term" }
```

Response format:
```json
{ "success": true, "message": "OK", "data": [...] }
```

### Installation

1. Build native messaging host:
   ```bash
   cd extensions/i-rs-native-msg
   cargo build --release
   ```

2. Install binary to PATH and create host manifest:
   ```bash
   # macOS
   ./extensions/install-macos.sh
   
   # Linux
   ./extensions/install-linux.sh
   ```

3. Load extension in browser:
   - Chrome: `chrome://extensions/` → Load unpacked
   - Firefox: `about:debugging` → Load Temporary Add-on

4. Update extension ID in host manifest

## CLI Tools Summary (70 tools)

| Tool | Description | Special Commands |
|------|-------------|-----------------|
| i-rs-server | Server management | suggest |
| i-rs-password | Password management | - |
| i-rs-bookmark | Bookmark management | - |
| i-rs-note | Note management | - |
| i-rs-domain | Domain expiry tracking | - |
| i-rs-remind | Event reminders | done |
| i-rs-weight | Weight tracking | chart, stats |
| i-rs-height | Height tracking | stats |
| i-rs-mood | Mood tracking | calendar |
| i-rs-sleep | Sleep tracking | stats |
| i-rs-todo | Todo tracking | done |
| i-rs-water | Water intake tracking | - |
| i-rs-step | Step counting | - |
| i-rs-dose | Medicine dosage | - |
| i-rs-cycle | Menstrual cycle | - |
| i-rs-sit | Sedentary reminder | - |
| i-rs-allergy | Allergy tracking | - |
| i-rs-cal | Calorie estimation | - |
| i-rs-fast | Fasting tracking | - |
| i-rs-exercise | Exercise tracking | stats |
| i-rs-run | Running records | plan, stats |
| i-rs-cycling | Cycling tracking | stats |
| i-rs-habit | Habit tracking | checkin, streak |
| i-rs-sub | Subscription tracking | - |
| i-rs-bestby | Best-by date tracking | - |
| i-rs-ledger | Accounting | - |
| i-rs-recur | Recurring expenses | - |
| i-rs-budget | Budget management | expense, stats |
| i-rs-invest | Investment tracking | stats |
| i-rs-debt | Debt management | pay, stats |
| i-rs-invoice | Invoice management | stats |
| i-rs-tax | Tax records | stats |
| i-rs-goal | Savings goals | deposit, milestone, stats |
| i-rs-kv | Key-value storage | - |
| i-rs-keys | API key management | - |
| i-rs-meal | Meal tracking | - |
| i-rs-pig | Craving tracking | - |
| i-rs-grocery | Grocery list | purchase, clear |
| i-rs-tick | Duration tracking | - |
| i-rs-spark | Inspiration capture | - |
| i-rs-want | Wish list | - |
| i-rs-gift | Gift planning | stats |
| i-rs-movie | Movie tracking | stats |
| i-rs-podcast | Podcast tracking | stats |
| i-rs-contact | Contact management | remind, stats |
| i-rs-car | Vehicle management | fuel, maintain, stats |
| i-rs-project | Project management | milestone, stats |
| i-rs-article | Article tracker | stats |
| i-rs-read | Reading tracker | stats |
| i-rs-quote | Quote collection | - |
| i-rs-snippet | Code snippet manager | - |
| i-rs-vocab | Vocabulary learning | quiz, stats |
| i-rs-birthday | Birthday tracking | stats |
| i-rs-event | Event management | stats |
| i-rs-time | Time tracking | start, stop, report, stats |
| i-rs-deploy | Deployment tracking | rollback, stats |
| i-rs-vision | Vision tracking | stats |
| i-rs-sheet | Bedsheet replacement | - |
| i-rs-toothbrush | Toothbrush replacement | - |
| i-rs-towel | Towel replacement | - |
| i-rs-bed | Mattress/pillow replacement | - |
| i-rs-ac | AC cleaning | - |
| i-rs-filter | Filter cleaning | - |
| i-rs-purify | Water purifier filter | - |
| i-rs-appliance | Appliance management | stats |
| i-rs-plant | Plant care | water, stats |
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

# Test core library
cargo test -p i-rs-core

# Test REST API server (32 integration tests)
cargo test -p i-rs-api

# Check for warnings (MUST be 0)
cargo check

# Full CI check
cargo clippy --workspace -- -D warnings
cargo fmt --all --check

# Dependency audit
cargo install cargo-deny && cargo deny check
```

## i-rs-core Shared Library

The `i-rs-core` crate provides shared functionality for all CLI tools:

```
crates/i-rs-core/src/
├── lib.rs                    # Public API exports
├── macro.rs                  # Macros: create_store!, skill_command!, exit_on_error!
├── storage/
│   └── mod.rs              # Storage<T>, filter_by_tag, HasTags
├── presentation/
│   ├── mod.rs              # print_error/success/header/warning + render_table
│   ├── output.rs           # JSON output formatting
│   └── theme.rs            # Customizable theme (theme.json)
└── utils/
    ├── date.rs             # parse_date(), parse_datetime()
    └── validation.rs       # validate_name/url/weight/amount (21 tests)
```

### i-rs-core Exports

```rust
// Storage
pub use i_rs_core::storage::{Storage, filter_by_tag, HasTags};

// Presentation
pub use i_rs_core::presentation::{
    print_error, print_header, print_success, print_warning, println_dimmed,
    render_table, OutputFormat, Theme,
};
pub use i_rs_core::presentation::output::{output_list, output_item, output_error};

// Utils
pub use i_rs_core::utils::date::{parse_date, parse_datetime};
pub use i_rs_core::utils::validation::{
    validate_name, validate_url, validate_weight, validate_amount, ValidationError,
};

// Macros (all via i_rs_core::macro_name!)
// - create_store!(XxxStore, "filename")
// - skill_command!("i-rs-crate-name")
// - exit_on_error!(result, json_bool)
```

### i-rs-core Quick Ref

| Item | Description |
|------|-------------|
| `Storage<T>` | Generic JSON file persistence |
| `create_store!` | Generates `load_store()` + `save_store()` |
| `render_table()` | Consistent table styling (cyan borders, green rows) |
| `skill_command!` | Generates `SkillCommand` (clap Subcommand) + `handle_skill(&SkillCommand) -> Result<()>` with info/search/teach/install/summary/content/raw subcommands |
| `exit_on_error!` | Unified error handling with JSON support |
| `print_error/success/header/warning` | Colored output helpers |
| `output_list/item/error` | JSON response formatting |
| `parse_date/parse_datetime` | Flexible date parsing |
| `validate_name/url/weight/amount` | Input validation |

## Crate Structure

Each CLI crate follows this pattern:
```
crates/i-rs-{name}/
├── src/
│   ├── main.rs           # CLI entry point: Cli::parse() + exit_on_error!
│   ├── commands/         # add, delete, get, list, update, example, skill
│   │   └── skill.rs     # ONE LINE: i_rs_core::skill_command!("i-rs-xxx");
│   ├── models/           # Data structs with serde + tabled + BTreeMap store
│   ├── storage/          # ONE LINE: i_rs_core::create_store!(XxxStore, "xxx");
│   └── presentation/     # render_table() + custom format functions
├── Cargo.toml            # Minimal deps (no dirs/serde_json unless needed)
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
authors.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
i-rs-core = { path = "../i-rs-core" }
clap.workspace = true
anyhow.workspace = true
serde.workspace = true
tabled.workspace = true
owo-colors.workspace = true
chrono.workspace = true
uuid.workspace = true        # if using UUIDs
keyring.workspace = true     # if storing passwords/keys
keyring-core.workspace = true
```

Note: Do NOT add `dirs` (comes through i-rs-core), `serde_json` (only if directly used), `tokio`/`reqwest` (not used).

### Step 3: Create Source Files

**storage/mod.rs** (one line):
```rust
use crate::models::XxxStore;
i_rs_core::create_store!(XxxStore, "xxx");
```

**commands/skill.rs** (one line):
```rust
i_rs_core::skill_command!("i-rs-xxx");
```

**main.rs** (error handling):
```rust
fn main() {
    let cli = Cli::parse();
    let format = if cli.json { OutputFormat::Json } else { OutputFormat::Table };
    i_rs_core::exit_on_error!(run(cli.command, format), cli.json);
}
```

**presentation/mod.rs** (table rendering):
```rust
pub fn format_table(rows: &[XxxRow]) -> String {
    i_rs_core::render_table(&rows)
}
```

Full file list:
- `src/models/mod.rs` - Entity struct + Row struct (Tabled) + Store (BTreeMap)
- `src/storage/mod.rs` - `create_store!` macro call
- `src/presentation/mod.rs` - `render_table()` + count printing
- `src/commands/mod.rs` - Module exports
- `src/commands/add.rs` - Add command
- `src/commands/delete.rs` - Delete command
- `src/commands/get.rs` - Get command
- `src/commands/list.rs` - List command
- `src/commands/update.rs` - Update command
- `src/commands/example.rs` - Example command
- `src/commands/skill.rs` - `skill_command!` macro call
- `src/commands/data.rs` - Data management (export/import/clear)
- `src/main.rs` - CLI parsing with `exit_on_error!`

### Step 4: Create README.md (REQUIRED!)

### Step 5: Create Docs (REQUIRED!)
- `docs/crates/i-rs-{name}/index.md`
- `docs/crates/i-rs-{name}/usage.md`
- `docs/crates/i-rs-{name}/examples.md`
- `docs/crates/i-rs-{name}/test.md`

### Step 6: Create Skills (REQUIRED!)
`skills/i-rs-{name}/SKILL.md` with YAML frontmatter.

### Step 7: Update Workspace Cargo.toml
Add to `members` array.

### Step 8: Update VitePress Config
Add sidebar entry in `docs/.vitepress/config.ts`.

### Step 9: Build and Verify
```bash
cargo check
# Must be 0 errors, 0 warnings
```

---

## ⚠️ IMPORTANT: Incomplete Crate Checklist

- [ ] `crates/i-rs-{name}/README.md` exists
- [ ] `docs/crates/i-rs-{name}/index.md` exists
- [ ] `docs/crates/i-rs-{name}/usage.md` exists
- [ ] `docs/crates/i-rs-{name}/examples.md` exists
- [ ] `docs/crates/i-rs-{name}/test.md` exists
- [ ] `skills/i-rs-{name}/SKILL.md` exists
- [ ] `docs/.vitepress/config.ts` has sidebar entry for this crate
- [ ] `Cargo.toml` workspace has this crate in members
- [ ] `storage/mod.rs` uses `create_store!` macro
- [ ] `commands/skill.rs` uses `skill_command!` macro
- [ ] `commands/data.rs` exists with Export/Import/Clear commands
- [ ] `main.rs` uses `exit_on_error!` for error handling
- [ ] Store uses `BTreeMap` (not `HashMap`)
- [ ] CRUD methods named `add_entry`/`remove_entry`/`get_entry`/`get_entry_mut`

## Key Conventions

- **Workspace deps**: All dependencies defined in root `Cargo.toml`, crates use `.workspace = true`
- **i-rs-core dependency**: All crates depend on `i-rs-core = { path = "../i-rs-core" }`
- **Passwords**: Always store in OS keychain (keyring crate), NEVER in JSON config
- **Data location**: `~/.config/i-rs/` (override with `CONFIG_DIR` env var)
- **Date handling**: chrono with `ts_seconds` serde format
- **Error handling**: `exit_on_error!` macro in main.rs, `anyhow::Result` elsewhere
- **CLI framework**: clap with derive macro, snake_case params auto-convert to kebab-case
- **Output**: `render_table()` for tables, `output_list/output_item` for JSON
- **JSON output**: All commands support `--json` global flag
- **JSON output scope**: All handlers use `format` param. `handle_add`/`handle_update`/`handle_get`/`handle_list` return entity data via `output_item`/`output_list`; `handle_delete` returns `{"success": true, "message": "..."}` via `serde_json::json!`. (API uses service layer directly, not CLI handlers.)
- **Storage**: `BTreeMap<String, Entity>` (NOT HashMap)
- **CRUD naming**: `add_entry`, `remove_entry`, `get_entry`, `get_entry_mut`
- **No unwrap()**: Use `expect("message")` or proper error handling
- **Cargo.lock**: MUST be committed (reproducible builds)
- **Build**: `cargo check` must show 0 errors and 0 warnings

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

### Store (BTreeMap)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyStore {
    pub entries: BTreeMap<String, MyEntity>,
}

impl MyStore {
    pub fn add_entry(&mut self, entry: MyEntity) {
        self.entries.insert(entry.name.clone(), entry);
    }
    pub fn remove_entry(&mut self, name: &str) -> Option<MyEntity> {
        self.entries.remove(name)
    }
    pub fn get_entry(&self, name: &str) -> Option<&MyEntity> {
        self.entries.get(name)
    }
    pub fn get_entry_mut(&mut self, name: &str) -> Option<&mut MyEntity> {
        self.entries.get_mut(name)
    }
}
```

### storage/mod.rs
```rust
use crate::models::MyStore;
i_rs_core::create_store!(MyStore, "my-entity");
```

Generated by `create_store!`:
- `load_store()` / `save_store()` — basic persistence
- `export_data()` — serialize store to JSON string
- `import_data(input)` — deserialize and save from JSON string
- `clear_data()` — reset store to default (empty)

### commands/data.rs
Each crate has a `data` subcommand (export/import/clear):
```rust
use clap::Subcommand;
use std::io::Read;

#[derive(Subcommand, Debug, Clone)]
pub enum DataCommand {
    Export,
    Import { file: Option<String> },
    Clear,
}

pub fn handle(command: &DataCommand) -> anyhow::Result<()> { ... }
```

### commands/skill.rs
```rust
i_rs_core::skill_command!("i-rs-my-entity");
```

Generated `SkillCommand` subcommands:
- `info`: Structured metadata
- `search <query>`: Search within skill content
- `teach`: Generate AI teaching document
- `install [--agent] [path]`: Install skill file
- `summary`: Show description
- `content`: Show body after frontmatter
- `raw`: Show raw SKILL.md (default)

### main.rs Error Handling
```rust
fn main() {
    let cli = Cli::parse();
    let format = if cli.json { OutputFormat::Json } else { OutputFormat::Table };
    i_rs_core::exit_on_error!(run(cli.command, format), cli.json);
}
```

### presentation/mod.rs Pattern
```rust
use crate::models::{Entity, EntityRow};

pub use i_rs_core::presentation::{print_error, print_header, print_success, print_warning, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item, output_error};

pub fn format_table(entities: &[&Entity]) -> String {
    let rows: Vec<EntityRow> = entities.iter().map(EntityRow::from_entity).collect();
    i_rs_core::render_table(&rows)
}

pub fn print_entity_count(count: usize) {
    println!("\n{} {} entities", "Total:".dimmed(), count.to_string().cyan());
}
```

## Global Commands (All Crates)

### example Command
```bash
i-rs-{name} example
```

### skill Command
```bash
i-rs-{name} skill          # Show raw skill document
i-rs-{name} skill summary  # Show summary (from SKILL.md description)
i-rs-{name} skill content  # Show content (after YAML frontmatter)
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
  "meta": { "count": 10, "filter": "work" }
}
```

**Item Response:**
```json
{ "success": true, "data": {...} }
```

**Error Response:**
```json
{
  "success": false,
  "error": { "code": "NOT_FOUND", "message": "Entry 'xxx' not found" }
}
```

## Input Validation

```rust
use i_rs_core::{validate_name, validate_url, validate_weight, ValidationError};

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
| `validate_amount` | > 0, ≤1 billion |

## Bug Prevention

### Store Loading Pattern (CORRECT)
```rust
let mut store = storage::load_store()?;
// ... use store ...
storage::save_store(&store)?;
```

### Store Loading Pattern (INCORRECT - BUG!)
```rust
let store = storage::load_store()?;
// ... later ...
let mut store = storage::load_store()?;  // BUG!
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
- `rust-toolchain.toml` - Pinned Rust toolchain
- `deny.toml` - cargo-deny license/advisory configuration
- `docs/.vitepress/config.ts` - Documentation sidebar config
- `.github/workflows/check.yml` - CI (cargo check + clippy + fmt)
- `.github/workflows/release.yml` - Release automation
- `crates/i-rs-api/src/update.rs` - Generic JSON merge/partial-update utility
- `crates/i-rs-mcp/src/main.rs` - MCP server with `make_mcp_tools!` macro and session loop
- `crates/i-rs-mcp/src/transport.rs` - JSON-RPC 2.0 stdio transport layer

## VitePress Documentation

Documentation at `docs/` uses VitePress. Each crate has 4 pages in `docs/crates/i-rs-{name}/`:
- `index.md` - Overview
- `usage.md` - Command reference
- `examples.md` - Detailed examples
- `test.md` - Test records

Skills provide specialized instructions and workflows for specific tasks.
Use the skill tool to load a skill when a task matches its description.
