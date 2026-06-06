# Dashboard Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract claw-core as a pure AI engine library, restructure claw into `serve` (main product) + `tui` (debug) subcommands, add multi-tenancy to storage, implement incremental persistence, and refactor the dashboard frontend.

**Architecture:** One binary (`claw`) with two subcommands (`serve` / `tui`), sharing a single `claw-core` lib. `claw serve` exposes a unified HTTP API (SSE chat + data CRUD) and serves the React SPA. All state in DB, TUI and Dashboard share data via the same storage backend. Multi-user isolation via `user_id` on sessions/stats/agents.

**Tech Stack:** Rust (axum, tokio, clap, claw-core lib), React 19 (Vite, wouter, CSS Modules), PostgreSQL/MySQL/MongoDB/SQLite storage backends.

---

### Task 1: Create claw-core crate skeleton

**Files:**
- Create: `crates/claw-core/Cargo.toml`
- Create: `crates/claw-core/src/lib.rs`
- Modify: `Cargo.toml` (root workspace)

- [ ] **Step 1: Create claw-core Cargo.toml**

```toml
[package]
name = "i-rs-claw-core"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
anyhow = { workspace = true }
async-trait = { workspace = true }
base64 = { workspace = true }
chrono = { workspace = true }
dirs = { workspace = true }
owo-colors = { workspace = true }
pulldown-cmark = { workspace = true }
reqwest = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tokio = { workspace = true }
toml = { workspace = true }
uuid = { workspace = true }
fastrand = "2"
tracing = "0.1"
unicode-width = "0.2"
futures-util = "0.3"
ratatui = "0.30"
crossterm = "0.29"
rmcp = { version = "1.7.0", optional = true }

[features]
default = ["mcp"]
mcp = ["rmcp"]
```

- [ ] **Step 2: Create minimal lib.rs**

```rust
//! i-rs-claw-core: AI agent engine library.
//! Provides AppCore, SessionManager, LLM providers, tools, storage, and memory.
//! Used by both claw-serve and claw-tui.

pub mod app;
pub mod config;
pub mod core;
pub mod llm;
pub mod mcp;
pub mod memory;
pub mod message;
pub mod plugin;
pub mod providers;
pub mod router;
pub mod semantic;
pub mod session;
pub mod skill_store;
pub mod stats;
pub mod storage;
pub mod tool_cache;
pub mod tools;
pub mod convstore;
pub mod error;
pub mod utils;
pub mod test_helpers;

// Only TUI-relevant modules stay in the binary crate:
// - tui/ (ratatui rendering)
// - ui/ (ratatui UI components)
// - completion.rs (terminal completions)
// - theme.rs (terminal theming)
```

- [ ] **Step 3: Register claw-core in workspace Cargo.toml**

In root `Cargo.toml`, add to `[workspace] members` after `crates/claw`:
```toml
"crates/claw-core",
```

- [ ] **Step 4: Add claw-core dependency to claw's Cargo.toml**

In `crates/claw/Cargo.toml`, add under `[dependencies]`:
```toml
i-rs-claw-core = { path = "../claw-core", default-features = false, features = ["mcp"] }
```

- [ ] **Step 5: Verify skeleton compiles**

```bash
cargo check -p i-rs-claw-core
```

Expected: 0 errors, 0 warnings (lib.rs will have unresolved module declarations — this is expected in this step).

- [ ] **Step 6: Commit**

```bash
git add crates/claw-core/ Cargo.toml crates/claw/Cargo.toml
git commit -m "feat(claw-core): create crate skeleton for AI engine library"
```

---

### Task 2: Move core modules from claw to claw-core

**Files (move from `crates/claw/src/` to `crates/claw-core/src/`):**
- `app.rs`
- `config.rs`
- `convstore.rs`
- `error.rs`
- `llm.rs`
- `mcp.rs`
- `memory.rs`
- `message/` (mod.rs + accumulator.rs)
- `plugin.rs`
- `router.rs`
- `semantic.rs`
- `session.rs`
- `skill_store.rs`
- `tool_cache.rs`
- `tools/` (all files)
- `utils.rs`
- `test_helpers.rs`
- `core/` (all files)
- `providers/` (all files)
- `stats/` (all files)
- `storage/` (all files)

- [ ] **Step 1: Copy files to claw-core**

```bash
cp crates/claw/src/app.rs crates/claw-core/src/app.rs
cp crates/claw/src/config.rs crates/claw-core/src/config.rs
cp crates/claw/src/convstore.rs crates/claw-core/src/convstore.rs
cp crates/claw/src/error.rs crates/claw-core/src/error.rs
cp crates/claw/src/llm.rs crates/claw-core/src/llm.rs
cp crates/claw/src/mcp.rs crates/claw-core/src/mcp.rs
cp crates/claw/src/memory.rs crates/claw-core/src/memory.rs
cp -r crates/claw/src/message/ crates/claw-core/src/message/
cp crates/claw/src/plugin.rs crates/claw-core/src/plugin.rs
cp crates/claw/src/router.rs crates/claw-core/src/router.rs
cp crates/claw/src/semantic.rs crates/claw-core/src/semantic.rs
cp crates/claw/src/session.rs crates/claw-core/src/session.rs
cp crates/claw/src/skill_store.rs crates/claw-core/src/skill_store.rs
cp crates/claw/src/tool_cache.rs crates/claw-core/src/tool_cache.rs
cp -r crates/claw/src/tools/ crates/claw-core/src/tools/
cp crates/claw/src/utils.rs crates/claw-core/src/utils.rs
cp crates/claw/src/test_helpers.rs crates/claw-core/src/test_helpers.rs
cp -r crates/claw/src/core/ crates/claw-core/src/core/
cp -r crates/claw/src/providers/ crates/claw-core/src/providers/
cp -r crates/claw/src/stats/ crates/claw-core/src/stats/
cp -r crates/claw/src/storage/ crates/claw-core/src/storage/
```

- [ ] **Step 2: Update all `crate::` references in claw-core to use the claw-core crate path**

All files in claw-core use `crate::` for internal references — no changes needed since they're now the root of their own crate.

- [ ] **Step 3: Update claw's remaining files to use `i_rs_claw_core::` instead of `crate::` for moved modules**

Example change in `crates/claw/src/main.rs`:
```rust
// Before:
use crate::app::Message;
use crate::config::Config;
use crate::core::AppCore;

// After:
use i_rs_claw_core::app::Message;
use i_rs_claw_core::config::Config;
use i_rs_claw_core::core::AppCore;
```

Apply the same pattern to all remaining claw source files:
- `src/main.rs`
- `src/cli/mod.rs`, `src/cli/config_wizard.rs`, `src/cli/tools_ui.rs`
- `src/tui/mod.rs`, `src/tui/main_loop.rs`, `src/tui/clipboard.rs`, `src/tui/reminders.rs`
- `src/tui/handlers/*.rs`
- `src/ui/**/*.rs`
- `src/dashboard/mod.rs`, `src/dashboard/routes.rs`, `src/dashboard/assets.rs`
- `src/gateway/mod.rs`, `src/gateway/telegram.rs`, `src/gateway/wechat.rs`

Use sed for bulk replacements:
```bash
find crates/claw/src -name '*.rs' -exec sed -i '' 's/crate::app::/i_rs_claw_core::app::/g' {} +
find crates/claw/src -name '*.rs' -exec sed -i '' 's/crate::config::/i_rs_claw_core::config::/g' {} +
find crates/claw/src -name '*.rs' -exec sed -i '' 's/crate::core::/i_rs_claw_core::core::/g' {} +
find crates/claw/src -name '*.rs' -exec sed -i '' 's/crate::llm::/i_rs_claw_core::llm::/g' {} +
find crates/claw/src -name '*.rs' -exec sed -i '' 's/crate::providers::/i_rs_claw_core::providers::/g' {} +
find crates/claw/src -name '*.rs' -exec sed -i '' 's/crate::tools::/i_rs_claw_core::tools::/g' {} +
find crates/claw/src -name '*.rs' -exec sed -i '' 's/crate::storage::/i_rs_claw_core::storage::/g' {} +
find crates/claw/src -name '*.rs' -exec sed -i '' 's/crate::session::/i_rs_claw_core::session::/g' {} +
find crates/claw/src -name '*.rs' -exec sed -i '' 's/crate::stats::/i_rs_claw_core::stats::/g' {} +
find crates/claw/src -name '*.rs' -exec sed -i '' 's/crate::message::/i_rs_claw_core::message::/g' {} +
find crates/claw/src -name '*.rs' -exec sed -i '' 's/crate::memory::/i_rs_claw_core::memory::/g' {} +
find crates/claw/src -name '*.rs' -exec sed -i '' 's/crate::mcp::/i_rs_claw_core::mcp::/g' {} +
find crates/claw/src -name '*.rs' -exec sed -i '' 's/crate::semantic::/i_rs_claw_core::semantic::/g' {} +
find crates/claw/src -name '*.rs' -exec sed -i '' 's/crate::router::/i_rs_claw_core::router::/g' {} +
find crates/claw/src -name '*.rs' -exec sed -i '' 's/crate::convstore::/i_rs_claw_core::convstore::/g' {} +
find crates/claw/src -name '*.rs' -exec sed -i '' 's/crate::plugin::/i_rs_claw_core::plugin::/g' {} +
find crates/claw/src -name '*.rs' -exec sed -i '' 's/crate::skill_store::/i_rs_claw_core::skill_store::/g' {} +
find crates/claw/src -name '*.rs' -exec sed -i '' 's/crate::tool_cache::/i_rs_claw_core::tool_cache::/g' {} +
find crates/claw/src -name '*.rs' -exec sed -i '' 's/crate::error::/i_rs_claw_core::error::/g' {} +
find crates/claw/src -name '*.rs' -exec sed -i '' 's/crate::utils::/i_rs_claw_core::utils::/g' {} +
find crates/claw/src -name '*.rs' -exec sed -i '' 's/crate::gateway::/i_rs_claw_core::gateway::/g' {} +
```

- [ ] **Step 4: Remove moved modules from claw's src/**

```bash
rm crates/claw/src/app.rs
rm crates/claw/src/config.rs
rm crates/claw/src/convstore.rs
rm crates/claw/src/error.rs
rm crates/claw/src/llm.rs
rm crates/claw/src/mcp.rs
rm crates/claw/src/memory.rs
rm -rf crates/claw/src/message/
rm crates/claw/src/plugin.rs
rm crates/claw/src/router.rs
rm crates/claw/src/semantic.rs
rm crates/claw/src/session.rs
rm crates/claw/src/skill_store.rs
rm crates/claw/src/tool_cache.rs
rm -rf crates/claw/src/tools/
rm crates/claw/src/utils.rs
rm crates/claw/src/test_helpers.rs
rm -rf crates/claw/src/core/
rm -rf crates/claw/src/providers/
rm -rf crates/claw/src/stats/
rm -rf crates/claw/src/storage/
```

- [ ] **Step 5: Remove moved module declarations from claw's main.rs**

In `crates/claw/src/main.rs`, remove the `mod` declarations for all moved modules:
```rust
// Keep only:
mod cli;
mod completion;
#[cfg(feature = "dashboard")]
mod dashboard;
mod theme;
mod tui;
mod ui;
mod gateway;
```

- [ ] **Step 6: Handle conditional compilation for dashboard feature**

In `crates/claw/src/main.rs`, the `dashboard` module and its imports from claw-core must be gated:
```rust
#[cfg(feature = "dashboard")]
mod dashboard;

#[cfg(feature = "dashboard")]
use i_rs_claw_core::core::AppCore;
```

The `dashboard` feature must be forwarded from claw to claw-core in claw's `Cargo.toml`:
```toml
[features]
dashboard = ["i-rs-claw-core/dashboard", "axum", "tower-http", "rust-embed", "mime_guess"]
```

And claw-core's `Cargo.toml` needs:
```toml
[features]
dashboard = ["dep:axum", "dep:tower-http", "dep:rust-embed", "dep:mime_guess"]
```

Wait—actually, `axum`, `tower-http`, etc. are only needed by `crates/claw/` (the server), not by `claw-core`. The dashboard routes use these but the routes are in `crates/claw/src/dashboard/`, not in claw-core. So claw-core does NOT need the dashboard feature. Only claw needs it.

Let me fix: claw-core's Cargo.toml should NOT have `axum`, `tower-http`, `rust-embed`, or `mime_guess`. Those stay in claw's Cargo.toml.

But `AppCore` (in claw-core) uses `SessionManager`, and the dashboard routes need `SessionManager` from claw-core. That's fine — `SessionManager` has no axum dependency. The dashboard routes create axum handlers that call `SessionManager` methods.

So the dependency tree:
- claw-core: no axum, no dashboard feature
- claw: depends on claw-core, has axum/dashboard feature, dashboard/ routes call claw-core types

- [ ] **Step 7: Build and fix compilation errors iteratively**

```bash
cargo check --workspace 2>&1 | head -100
```

Iterate: fix error → rebuild → repeat until 0 errors, 0 warnings.

Common issues to fix:
- Missing `use` statements in claw for moved types
- `#[cfg(test)]` mod references needing full paths
- `crate::test_helpers` → `i_rs_claw_core::test_helpers`

- [ ] **Step 8: Run existing tests to verify nothing broke**

```bash
cargo test -p i-rs-claw-core -- --test-threads=1
cargo test -p i-rs-claw -- --test-threads=1
```

Expected: all 206 tests pass (or close to it; some test imports may need path updates).

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "refactor: extract claw-core as pure AI engine library

- Move app, config, core, providers, tools, storage, session, stats,
  memory, message, mcp, semantic, router, and related modules to claw-core
- Update claw binary crate to depend on claw-core
- Keep TUI rendering (tui/, ui/) and dashboard HTTP layer in claw
- Conditionally compile dashboard module under feature flag
- All existing tests continue to pass"
```

---

### Task 3: Restructure claw binary — serve + tui CLI

**Files:**
- Modify: `crates/claw/src/main.rs`
- Create: `crates/claw/src/lib.rs`
- Move: `crates/claw/src/cli/config_wizard.rs` → TUI code stays in claw
- Modify: `crates/claw/src/gateway/mod.rs` — gateway now uses serve mode API internally

- [ ] **Step 1: Split main.rs into lib.rs + main.rs**

Create `crates/claw/src/lib.rs`:
```rust
// Re-export claw-core for convenience when used as a lib
pub use i_rs_claw_core::*;
```

Modify `crates/claw/src/main.rs` — add `serve` subcommand and make `dashboard` a flag on it:
```rust
use clap::{Parser, Subcommand};
use i_rs_claw_core::config::Config;

#[derive(Parser)]
#[command(name = "i-rs-claw", version, about = "AI Personal Assistant")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Start HTTP API server + Web Dashboard (default mode)
    Serve {
        /// Host to bind (default: 0.0.0.0)
        #[arg(long, default_value = "0.0.0.0")]
        host: String,
        /// Port to listen on (default: 3000)
        #[arg(long, short, default_value = "3000")]
        port: u16,
        /// Disable the Web Dashboard UI (API only)
        #[arg(long)]
        api_only: bool,
    },
    /// Start TUI terminal interface (debug/power-user mode)
    Tui {
        /// Session ID to resume
        #[arg(long)]
        session: Option<String>,
        /// User ID for multi-tenant mode
        #[arg(long, env = "CLAW_USER")]
        user: Option<String>,
    },
    /// Non-interactive single-turn chat
    Ask {
        /// Message to send
        message: String,
        /// Session ID (creates new if omitted)
        #[arg(long)]
        session: Option<String>,
        /// Agent ID
        #[arg(long)]
        agent: Option<String>,
    },
    /// Manage sessions (list, export)
    Session {
        #[arg(long)]
        list: bool,
        #[arg(long)]
        export_md: Option<String>,
        #[arg(long)]
        export_json: Option<String>,
    },
    /// Start social platform gateway (Telegram/WeChat)
    Gateway,
    /// Token usage statistics
    Stats {
        #[arg(long, default_value = "today")]
        period: String,
        #[arg(long)]
        json: bool,
    },
    /// Plugin management
    Plugin {
        #[arg(long)]
        list: bool,
        #[arg(long)]
        info: Option<String>,
        #[arg(long)]
        enable: Option<String>,
        #[arg(long)]
        disable: Option<String>,
    },
    /// Skill management
    Skill {
        #[arg(long)]
        list: bool,
        #[arg(long)]
        install: Option<String>,
        #[arg(long)]
        remove: Option<String>,
        #[arg(long)]
        info: Option<String>,
    },
    /// Interactive configuration
    Config,
}

#[tokio::main]
async fn main() {
    // ... tracing init (unchanged) ...

    let cli = Cli::parse();

    match cli.command.unwrap_or(Command::Serve {
        host: "0.0.0.0".to_string(),
        port: 3000,
        api_only: false,
    }) {
        Command::Serve { host, port, api_only } => {
            // Load config, build AppCore, start HTTP server
            let config = Config::load().expect("Failed to load config");
            let app_core = AppCore::new(config.clone()).await;
            serve::run(app_core, host, port, api_only).await;
        }
        Command::Tui { session, user } => {
            let config = Config::load().expect("Failed to load config");
            let app_core = AppCore::new(config).await;
            tui::run(app_core, session, user);
        }
        // ... other commands (unchanged structure, using claw-core) ...
    }
}
```

- [ ] **Step 2: Create serve module**

Create `crates/claw/src/serve/mod.rs`:
```rust
mod routes;

use i_rs_claw_core::core::AppCore;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub core: Arc<RwLock<AppCore>>,
    pub auth_token: String,
}

impl AppState {
    pub fn new(core: AppCore, auth_token: String) -> Self {
        Self { core: Arc::new(RwLock::new(core)), auth_token }
    }
}

pub async fn run(core: AppCore, host: String, port: u16, api_only: bool) {
    use axum::Router;

    let auth_token = generate_or_load_token()?;

    let state = AppState::new(core, auth_token.clone());

    let auth_middleware = axum::middleware::from_fn_with_state(state.clone(), auth_guard);

    let api_routes = Router::new()
        .route("/api/health", axum::routing::get(routes::health))
        .route("/api/chat", axum::routing::post(routes::chat))
        .route("/api/chat/stream/{session_id}", axum::routing::get(routes::chat_stream_resume))
        .route("/api/sessions", axum::routing::get(routes::list_sessions).post(routes::create_session))
        .route("/api/sessions/current", axum::routing::get(routes::get_current_session))
        .route("/api/sessions/{id}", axum::routing::get(routes::get_session).delete(routes::delete_session))
        .route("/api/sessions/{id}/switch", axum::routing::post(routes::switch_session))
        .route("/api/sessions/{id}/feedback", axum::routing::post(routes::post_session_feedback))
        .route("/api/agents", axum::routing::get(routes::get_agents).post(routes::create_agent))
        .route("/api/agents/{id}", axum::routing::get(routes::get_agent_detail).put(routes::update_agent).delete(routes::delete_agent))
        .route("/api/stats", axum::routing::get(routes::get_stats))
        .route("/api/tools", axum::routing::get(routes::list_tools))
        .route("/api/skills", axum::routing::get(routes::list_skills))
        .route("/api/plugins", axum::routing::get(routes::list_plugins))
        .route("/api/config", axum::routing::get(routes::get_config).patch(routes::update_config))
        .route("/api/providers", axum::routing::get(routes::list_providers))
        .route("/api/data/{tool}", axum::routing::get(routes::data_list).post(routes::data_create))
        .route("/api/data/{tool}/{id}", axum::routing::get(routes::data_get).delete(routes::data_delete).patch(routes::data_update))
        // Images, checkpoints, memory, evals, guardrails — same as existing
        .route("/api/images/{filename}", axum::routing::get(routes::serve_image))
        .route("/api/checkpoints", axum::routing::get(routes::list_checkpoints))
        .route("/api/checkpoints/{id}", axum::routing::get(routes::get_checkpoint_detail))
        .route("/api/checkpoints/restore", axum::routing::post(routes::restore_checkpoint))
        .route("/api/memory/layered", axum::routing::get(routes::get_layered_memory).post(routes::clear_layered_memory))
        .route("/api/memory/search", axum::routing::get(routes::search_layered_memory))
        .route("/api/evals", axum::routing::get(routes::run_evals))
        .route("/api/guardrails/check", axum::routing::post(routes::check_guardrails))
        .layer(auth_middleware);

    let app = if api_only {
        api_routes.with_state(state)
    } else {
        let static_routes = Router::new()
            .route("/", axum::routing::get(crate::dashboard::assets::serve_root))
            .route("/{*path}", axum::routing::get(crate::dashboard::assets::serve_assets));
        Router::new()
            .merge(api_routes)
            .merge(static_routes)
            .with_state(state)
    };

    let addr: SocketAddr = format!("{}:{}", host, port).parse().expect("Invalid address");
    println!("  Dashboard: http://{}#{}", addr, auth_token);

    let listener = tokio::net::TcpListener::bind(addr).await.expect("Failed to bind");
    axum::serve(listener, app).await.expect("Server error");
}
```

- [ ] **Step 3: Move old dashboard/ to serve/routes.rs**

Copy `crates/claw/src/dashboard/routes.rs` → `crates/claw/src/serve/routes.rs` and update imports.

The existing `dashboard/mod.rs` Auth logic moves to `serve/mod.rs`. The `dashboard/assets.rs` stays in `crates/claw/src/dashboard/assets.rs` (embedded SPA serving is static and doesn't need restructuring now).

Remove old `crates/claw/src/dashboard/mod.rs` (content merged into `serve/mod.rs`).

- [ ] **Step 4: Update TUI entry to accept AppCore as parameter**

In `crates/claw/src/tui/mod.rs`, change `pub fn run(session: Option<String>)` to:
```rust
pub fn run(core: i_rs_claw_core::core::AppCore, session: Option<String>, user: Option<String>) {
    // Use the passed-in core instead of creating a new one
    // Store user_id from the --user flag or CLAW_USER env
}
```

- [ ] **Step 5: Move gateway to use serve-mode internally**

In `crates/claw/src/gateway/mod.rs`, the gateway currently creates its own AppCore. Change to accept AppCore from main instead:
```rust
pub async fn run(core: i_rs_claw_core::core::AppCore) {
    // Use passed-in core
}
```

- [ ] **Step 6: Build and fix**

```bash
cargo check --workspace 2>&1 | head -50
```

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "refactor(claw): restructure binary with serve (default) + tui subcommands

- `claw serve` is now the default mode (HTTP API + Web Dashboard)
- `claw tui` is the debug/power-user mode
- Gateway and other subcommands use shared AppCore from main
- Serve module extracted from dashboard/ for cleaner separation"
```

---

### Task 4: Add user_id to storage layer and session model

**Files:**
- Modify: `crates/claw-core/src/session.rs` — add `user_id` to `SessionMeta`
- Modify: `crates/claw-core/src/stats/mod.rs` — add `user_id` to `TokenRecord`
- Modify: `crates/claw-core/src/storage/file.rs` — handle `user_id` in JSON
- Modify: `crates/claw-core/src/storage/sql/sqlite.rs` — ALTER TABLE
- Modify: `crates/claw-core/src/storage/sql/mysql.rs` — ALTER TABLE
- Modify: `crates/claw-core/src/storage/sql/postgres.rs` — ALTER TABLE
- Modify: `crates/claw-core/src/storage/mongo.rs` — add field
- Modify: `crates/claw-core/src/storage/redis.rs` — add field
- Modify: `crates/claw-core/src/config.rs` — add `DashboardUsersConfig`

- [ ] **Step 1: Add user_id to SessionMeta**

In `crates/claw-core/src/session.rs`, modify `SessionMeta`:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMeta {
    pub id: String,
    pub title: String,
    pub agent_id: String,
    pub user_id: String,  // NEW — default: "default"
    pub state: SessionState,
    pub created_at: i64,
    pub updated_at: i64,
    pub message_count: usize,
}
```

- [ ] **Step 2: Add user_id to TokenRecord**

In `crates/claw-core/src/stats/mod.rs`, modify `TokenRecord`:
```rust
pub struct TokenRecord {
    pub id: String,
    pub timestamp: i64,
    pub user_id: String,  // NEW — default: "default"
    pub agent_id: String,
    pub model: String,
    // ... rest unchanged
}
```

- [ ] **Step 3: Update SQL DDL**

In `crates/claw-core/src/storage/sql/sqlite.rs`, in the `create_tables` SQL:
```sql
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL DEFAULT '',
    agent_id TEXT NOT NULL DEFAULT 'default',
    user_id TEXT NOT NULL DEFAULT 'default',
    state TEXT NOT NULL DEFAULT 'Active',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    message_count INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS token_records (
    id TEXT PRIMARY KEY,
    timestamp INTEGER NOT NULL,
    user_id TEXT NOT NULL DEFAULT 'default',
    agent_id TEXT NOT NULL,
    model TEXT NOT NULL,
    -- ... rest unchanged
);
```

Same pattern for MySQL and Postgres (`user_id VARCHAR(255) NOT NULL DEFAULT 'default'`).

- [ ] **Step 4: Update file backend**

In `crates/claw-core/src/storage/file.rs`, `SessionMeta` serialization already includes `user_id` via serde. For `TokenRecord`, same — serde handles it.

Add migration logic: when loading old sessions/stats that lack `user_id`, default it to `"default"`:
```rust
// In load_all():
for mut session in sessions {
    if session.user_id.is_empty() {
        session.user_id = "default".to_string();
    }
}
```

- [ ] **Step 5: Update MongoDB backend**

In `crates/claw-core/src/storage/mongo.rs`, add `user_id` field to documents and create index:
```rust
// In session document:
doc! { "user_id": session.user_id, ... }

// Index:
collection.create_index(
    doc! { "user_id": 1 },
).await?;
```

- [ ] **Step 6: Update Redis backend**

In `crates/claw-core/src/storage/redis.rs`, include `user_id` in the JSON stored in hash fields:
```rust
let json = serde_json::json!({
    "id": meta.id,
    "user_id": meta.user_id,
    // ...
});
```

- [ ] **Step 7: Add DashboardUsersConfig**

In `crates/claw-core/src/config.rs`, add:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardUser {
    pub id: String,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub auth_token: Option<String>,
    pub users: Vec<DashboardUser>,  // NEW: multi-user support
}
```

- [ ] **Step 8: Update SessionManager to accept/filter by user_id**

In `crates/claw-core/src/session.rs`, add `user_id` parameter to key methods:
```rust
impl SessionManager {
    pub fn create_session_for(&mut self, agent_id: &str, user_id: &str) -> String {
        // Same as before but sets meta.user_id = user_id.to_string()
    }

    pub fn sessions_for_user(&self, user_id: &str) -> Vec<&SessionMeta> {
        self.sessions.values().filter(|s| s.user_id == user_id).collect()
    }
}
```

- [ ] **Step 9: Build and verify**

```bash
cargo check --workspace
```

- [ ] **Step 10: Commit**

```bash
git add -A
git commit -m "feat(storage): add user_id to sessions, stats, and config for multi-tenancy

- SessionMeta, TokenRecord now have user_id (default: 'default')
- All storage backends (SQLite/MySQL/Postgres/Mongo/Redis/File) updated
- DashboardConfig supports [[dashboard.users]] for token→user_id mapping
- SessionManager API extended with user_id parameter (backward compatible)"
```

---

### Task 5: Implement incremental chat persistence (serve-side)

**Files:**
- Modify: `crates/claw/src/serve/routes.rs` — rewrite `chat()` and `chat_stream_resume()`
- Modify: `crates/claw-core/src/core/engine/mod.rs` — add per-round persistence callback

- [ ] **Step 1: Refactor POST /api/chat to single endpoint SSE**

In `crates/claw/src/serve/routes.rs`, replace `send_message` + `chat_stream` with `chat`:
```rust
/// POST /api/chat — send message and receive SSE stream
pub async fn chat(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> axum::response::Response {
    let text = match body.get("message").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => return ApiResponse::err("Missing 'message'").into_response(),
    };
    let agent_id = body.get("agent_id").and_then(|v| v.as_str()).unwrap_or("default").to_string();
    let user_id = resolve_user_id(&state); // from auth token

    // 1. Get or create session
    let mut core = state.core.write().await;
    let sid = core.session_mgr
        .get_or_create_session_for(&agent_id, &user_id);

    // 2. Persist user message immediately
    let user_msg = i_rs_claw_core::app::Message::User { text: text.clone() };
    core.session_mgr.persist_messages(&sid, &[user_msg]);

    let records = core.session_mgr.load_app_messages(&sid, 50);
    let msgs = core.build_messages_from_log(&records, &agent_id);
    let recent: Vec<Value> = records.iter()
        .filter_map(|m| match m {
            i_rs_claw_core::app::Message::User { text } =>
                Some(serde_json::json!({"role":"user","content":text})),
            i_rs_claw_core::app::Message::Assistant { text, .. } if !text.is_empty() =>
                Some(serde_json::json!({"role":"assistant","content":text})),
            _ => None,
        }).collect();

    // 3. Create event channel
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    // 4. Spawn chat_loop with incremental persistence
    let persist_core = state.core.clone();
    let persist_sid = sid.clone();
    core.spawn_chat_for_async(tx, msgs, &agent_id, &recent);
    drop(core);

    // 5. Return SSE stream with incremental persistence
    let stream = async_stream::stream! {
        let mut rx = rx;
        let mut event_seq: u64 = 0;

        while let Some(event) = rx.recv().await {
            match &event {
                i_rs_claw_core::llm::LlmEvent::Token(t) => {
                    let sse = Event::default().event("token").data(t.clone()).id(event_seq.to_string());
                    event_seq += 1;
                    yield Ok::<_, Infallible>(sse);
                }
                i_rs_claw_core::llm::LlmEvent::NewRound => {
                    // Persist current assistant response NOW (incremental!)
                    let mut core = persist_core.write().await;
                    // The accumulator has the current assistant text + tool calls
                    // We need to extract and persist just the completed round
                    if let Err(e) = persist_current_round(&mut core, &persist_sid) {
                        tracing::error!("incremental persist failed: {}", e);
                    }
                    drop(core);
                    let sse = Event::default().event("new_round").id(event_seq.to_string()).data("");
                    event_seq += 1;
                    yield Ok::<_, Infallible>(sse);
                }
                i_rs_claw_core::llm::LlmEvent::ToolExecuted { name, args, result, step, total_steps } => {
                    let data = serde_json::json!({
                        "name": name, "args": args, "result": result,
                        "step": step, "total_steps": total_steps,
                    });
                    let sse = Event::default().event("tool_executed").id(event_seq.to_string()).data(data.to_string());
                    event_seq += 1;
                    yield Ok::<_, Infallible>(sse);
                }
                i_rs_claw_core::llm::LlmEvent::Done(msgs, usage, _trace_id) => {
                    // Final persistence + quality
                    let mut core = persist_core.write().await;
                    core.session_mgr.save_api_messages(&persist_sid, &msgs);
                    let quality = core.evaluate_completed_session(&persist_sid);
                    // ... persist quality ...
                    drop(core);
                    let done_data = serde_json::json!({"usage": usage, "quality": quality, "cursor": event_seq});
                    let sse = Event::default().event("done").id(event_seq.to_string()).data(done_data.to_string());
                    yield Ok::<_, Infallible>(sse);
                    break;
                }
                i_rs_claw_core::llm::LlmEvent::Error(e) => {
                    let sse = Event::default().event("error").id(event_seq.to_string()).data(e.clone());
                    yield Ok::<_, Infallible>(sse);
                    break;
                }
                _ => {}
            }
        }
    };

    Sse::new(stream).into_response()
}
```

- [ ] **Step 2: Write helper for round persistence**

```rust
fn persist_current_round(core: &mut AppCore, session_id: &str) -> anyhow::Result<()> {
    // The chat_loop has emitted a NewRound event, meaning the assistant's
    // response (text + tool calls) for this round is complete.
    // The accumulator in the stream fold already has the complete set.
    // We need to extract from the accumulator's internal state.
    //
    // Since the accumulator is in the stream fold state (not accessible here),
    // we need an alternative approach: have chat_loop emit the completed
    // assistant message as part of the NewRound event.
    //
    // NEW APPROACH: Add a field to LlmEvent::NewRound with the completed message:
    Ok(())
}
```

Actually, the better approach is to **modify `LlmEvent::NewRound`** to carry the completed messages. Let me redesign.

- [ ] **Step 3: Enhance LlmEvent::NewRound to carry persistable data**

In `crates/claw-core/src/llm.rs`:
```rust
pub enum LlmEvent {
    // ... existing variants ...
    /// A ReAct round completed. Carries the assistant response + tool calls for this round.
    NewRound {
        /// The assistant's text response (may be empty if tool-only round)
        assistant_text: String,
        /// Tool calls executed in this round (name, args, result)
        tool_calls: Vec<ToolCallResult>,
    },
    // ...
}

#[derive(Debug, Clone)]
pub struct ToolCallResult {
    pub name: String,
    pub args: String,
    pub result: String,
    pub step: u32,
    pub total_steps: u32,
}
```

- [ ] **Step 4: Update chat_loop to emit enhanced NewRound**

In `crates/claw-core/src/core/engine/mod.rs`, find where `LlmEvent::NewRound` is emitted and change to include the round data:
```rust
// Before:
tx.send(LlmEvent::NewRound)?;

// After:
tx.send(LlmEvent::NewRound {
    assistant_text: accumulated_text.clone(),
    tool_calls: round_tool_calls.clone(),
})?;
```

- [ ] **Step 5: Use NewRound data for incremental persistence**

Now in the serve route:
```rust
LlmEvent::NewRound { assistant_text, tool_calls } => {
    let mut core = persist_core.write().await;
    let mut messages = Vec::new();
    if !assistant_text.is_empty() {
        messages.push(i_rs_claw_core::app::Message::Assistant {
            text: assistant_text.clone(),
            reasoning: String::new(),
        });
    }
    for tc in &tool_calls {
        messages.push(i_rs_claw_core::app::Message::ToolCall {
            name: tc.name.clone(),
            args: tc.args.clone(),
            result: tc.result.clone(),
            step: tc.step,
            total_steps: tc.total_steps,
        });
    }
    if let Err(e) = core.session_mgr.persist_messages(&persist_sid, &messages) {
        tracing::error!("incremental persist failed: {}", e);
    }
    drop(core);
    // ... emit SSE new_round as before ...
}
```

- [ ] **Step 6: Build and fix**

```bash
cargo check --workspace
```

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat(serve): incremental chat persistence — persist per ReAct round

- LlmEvent::NewRound now carries assistant_text and tool_calls
- POST /api/chat is single-endpoint SSE (no separate stream endpoint)
- Each chat_loop round persists immediately to DB
- SSE disconnection no longer causes message loss"
```

---

### Task 6: Auth middleware — token → user_id resolution

**Files:**
- Modify: `crates/claw/src/serve/mod.rs` — update auth guard

- [ ] **Step 1: Add user_id resolution to auth guard**

```rust
async fn auth_guard(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    let provided = req.headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    let user_id = match provided {
        Some(token) => {
            // Check user-specific tokens first
            let core = state.core.read().await;
            if let Some(user) = core.config.dashboard.users.iter()
                .find(|u| u.token == token)
            {
                user.id.clone()
            } else if Some(token) == state.auth_token.as_deref() {
                // Fallback to global token → "default" user
                "default".to_string()
            } else {
                return unauthorized_response(req.uri().path());
            }
        }
        None => return unauthorized_response(req.uri().path()),
    };

    // Inject user_id into request extensions
    req.extensions_mut().insert(UserId(user_id));
    next.run(req).await
}

#[derive(Clone)]
struct UserId(String);

// Helper for handlers
fn resolve_user_id(state: &AppState) -> String {
    // For handlers that can't use Extensions directly, default to "default"
    // (Extensions-based resolution is preferred where possible)
    "default".to_string()
}
```

- [ ] **Step 2: Update all route handlers to extract user_id**

Add an axum extractor:
```rust
use axum::extract::FromRequestParts;

#[async_trait]
impl<S> FromRequestParts<S> for UserId
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut axum::http::request::Parts, _: &S) -> Result<Self, Self::Rejection> {
        parts.extensions.get::<UserId>()
            .cloned()
            .ok_or((StatusCode::UNAUTHORIZED, "Missing user context"))
    }
}
```

Then in route signatures:
```rust
pub async fn list_sessions(
    State(state): State<AppState>,
    UserId(user_id): UserId,
) -> Json<ApiResponse<Vec<Value>>> {
    let core = state.core.read().await;
    let sessions = core.session_mgr.sessions_for_user(&user_id);
    // ...
}
```

- [ ] **Step 3: Build and fix**

```bash
cargo check --workspace
```

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(serve): auth middleware resolves token to user_id

- Auth guard checks [[dashboard.users]] for multi-user tokens
- Global auth_token maps to 'default' user
- UserId extractor available in all route handlers
- Sessions filtered by user_id"
```

---

### Task 7: Frontend — add routing, split Chat.tsx, CSS Modules

**Files:**
- Modify: `crates/claw/dashboard-ui/src/App.tsx`
- Create: `crates/claw/dashboard-ui/src/hooks/useChatStream.ts`
- Create: `crates/claw/dashboard-ui/src/hooks/useSessionLoader.ts`
- Create: `crates/claw/dashboard-ui/src/components/ChatInput.tsx`
- Create: `crates/claw/dashboard-ui/src/components/MessageList.tsx`
- Create: `crates/claw/dashboard-ui/src/components/StreamingBubble.tsx`
- Create: `crates/claw/dashboard-ui/src/components/SessionCard.tsx`
- Create: `crates/claw/dashboard-ui/src/components/ChatInput.module.css`
- Create: `crates/claw/dashboard-ui/src/components/MessageList.module.css`
- Create: `crates/claw/dashboard-ui/src/components/StreamingBubble.module.css`
- Modify: `crates/claw/dashboard-ui/src/pages/Chat.tsx` (simplify to use hooks)
- Modify: `crates/claw/dashboard-ui/src/api.ts` (single endpoint)
- Install: wouter package

- [ ] **Step 1: Install wouter**

```bash
cd crates/claw/dashboard-ui && pnpm add wouter
```

- [ ] **Step 2: Add routing to App.tsx**

```tsx
import { Route, Switch, useLocation } from 'wouter'

// Replace current page state with wouter routes:
export default function App() {
  const [location, setLocation] = useLocation()
  if (!authenticated) return <TokenPrompt onSubmit={handleTokenSubmit} />

  return (
    <div className="app-layout">
      <aside className="sidebar">
        {/* ... sidebar unchanged ... */}
        {NAV_ITEMS.map(item => (
          <button
            key={item.id}
            className={`nav-item${location === `/${item.id === 'chat' ? '' : item.id}` ? ' active' : ''}`}
            onClick={() => setLocation(item.id === 'chat' ? '/' : `/${item.id}`)}
          >
            {item.label}
          </button>
        ))}
      </aside>
      <main className="main-content">
        <Switch>
          <Route path="/" component={ChatPage} />
          <Route path="/sessions" component={SessionsPage} />
          <Route path="/agents" component={AgentsPage} />
          <Route path="/usage" component={UsagePage} />
          <Route path="/settings" component={ConfigPage} />
          <Route path="/tools" component={ToolsPage} />
          <Route path="/plugins" component={PluginsPage} />
          <Route path="/skills" component={SkillsPage} />
        </Switch>
      </main>
    </div>
  )
}
```

- [ ] **Step 3: Create useChatStream hook**

`crates/claw/dashboard-ui/src/hooks/useChatStream.ts`:
```typescript
import { useRef, useState, useCallback, useEffect } from 'react'
import { authFetch, type ToolCallMsg, type TokenUsage, type QualityScore, type ImageGeneratedEvent, type EvaluationEvent } from '../api'

const BASE = window.location.pathname.replace(/\/$/, '') + '/api'

interface ChatStreamState {
  content: string
  reasoning: string
  toolCalls: ToolCallMsg[]
  usage: TokenUsage | null
  quality: QualityScore | null
  error: string | null
  done: boolean
}

export function useChatStream() {
  const [streaming, setStreaming] = useState(false)
  const [state, setState] = useState<ChatStreamState>({
    content: '', reasoning: '', toolCalls: [],
    usage: null, quality: null, error: null, done: false,
  })
  const abortRef = useRef<AbortController | null>(null)
  const cursorRef = useRef(0)
  const stateRef = useRef(state)
  stateRef.current = state

  const abort = useCallback(() => {
    abortRef.current?.abort()
    setStreaming(false)
  }, [])

  const sendMessage = useCallback(async (message: string, agentId?: string) => {
    abortRef.current?.abort()
    const controller = new AbortController()
    abortRef.current = controller

    setState({ content: '', reasoning: '', toolCalls: [], usage: null, quality: null, error: null, done: false })
    setStreaming(true)

    try {
      const resp = await authFetch('/chat', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ message, agent_id: agentId }),
        signal: controller.signal,
      })

      const reader = resp.body?.getReader()
      if (!reader) throw new Error('No response body')

      const decoder = new TextDecoder()
      let buffer = ''

      while (true) {
        const { done, value } = await reader.read()
        if (done) break

        buffer += decoder.decode(value, { stream: true })
        const lines = buffer.split('\n')
        buffer = lines.pop() || ''

        for (const line of lines) {
          const trimmed = line.trim()
          if (!trimmed.startsWith('data: ')) continue
          const data = trimmed.slice(6)
          const eventType = line.includes('event: ') ? '' : 'token'
          // Parse SSE event and update stateRef accordingly
          // ... (token/reasoning/tool_executed/new_round/done/error handling,
          //      persisting completed rounds as separate messages)
        }
      }
    } catch (err: any) {
      if (err.name !== 'AbortError') {
        setState(prev => ({ ...prev, error: String(err) }))
      }
    } finally {
      setStreaming(false)
    }
  }, [])

  const resumeStream = useCallback(async (sessionId: string) => {
    // Reconnect SSE from cursor position
    const controller = new AbortController()
    abortRef.current = controller
    setStreaming(true)
    // ... connect to GET /api/chat/stream/{sessionId}?cursor={cursorRef.current}
  }, [])

  return { streaming, state, sendMessage, resumeStream, abort }
}
```

- [ ] **Step 4: Create useSessionLoader hook**

`crates/claw/dashboard-ui/src/hooks/useSessionLoader.ts`:
```typescript
import { useState, useEffect, useCallback } from 'react'
import { getCurrentSession, listSessions, createSession, switchSession, type ChatMessage, type ToolCallMsg } from '../api'

function deserializeMessages(raw: any[]): ChatMessage[] {
  const msgs: ChatMessage[] = []
  let pendingToolCalls: ToolCallMsg[] = []
  for (const m of raw) {
    if (m.role === 'user') {
      pendingToolCalls = []
      msgs.push({ role: 'user', content: m.content || '' })
    } else if (m.role === 'assistant') {
      msgs.push({
        role: 'assistant',
        content: m.content || '',
        reasoning: m.reasoning || undefined,
        toolCalls: pendingToolCalls.length > 0 ? [...pendingToolCalls] : undefined,
      })
      pendingToolCalls = []
    } else if (m.role === 'tool_call') {
      pendingToolCalls.push({
        name: m.name || '', args: m.args || '', result: m.result || '',
        step: 0, total_steps: 1,
      })
    } else if (m.role === 'image') {
      msgs.push({
        role: 'image', content: '',
        image: { path: m.path, alt_text: m.alt_text, width: m.width, height: m.height, format: m.format, url: `/api/images/${m.path}` },
      })
    } else if (m.role === 'evaluation') {
      msgs.push({ role: 'evaluation', content: m.content || '', evaluation: { tool: m.tool, valid: m.valid, issues: m.issues || [] } })
    } else if (m.role === 'quality') {
      msgs.push({ role: 'quality', content: m.content || '', quality: { score: m.score || '0', complete: m.complete, issues: m.issues || [], references_valid: m.references_valid } })
    } else if (m.role === 'feedback') {
      msgs.push({ role: 'feedback', content: m.content || '', feedback: { positive: m.positive, message: m.message || '' } })
    }
  }
  return msgs
}

export function useSessionLoader(selectedAgent: string) {
  const [messages, setMessages] = useState<ChatMessage[]>([])
  const [sessionId, setSessionId] = useState<string | null>(null)
  const [sessionTitle, setSessionTitle] = useState('')
  const [sessionAgent, setSessionAgent] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)

  const load = useCallback(async () => {
    setLoading(true)
    // Single unified deserialize function — no more duplicated code
    const resp = await getCurrentSession()
    if (resp.success && resp.data?.id && (resp.data.agent_id || 'default') === selectedAgent) {
      setSessionId(resp.data.id)
      setSessionTitle(resp.data.title || 'Untitled')
      setSessionAgent(resp.data.agent_id || null)
      setMessages(deserializeMessages(resp.data.messages || []))
    } else {
      // Try matching agent session or create new
      const sessionsResp = await listSessions()
      const agentSessions = (sessionsResp.data || [])
        .filter((s: any) => (s.agent_id || 'default') === selectedAgent)
        .sort((a: any, b: any) => b.created_at - a.created_at)
      if (agentSessions.length > 0) {
        await switchSession(agentSessions[0].id)
        // Recursive call to load with correct session
        await load()
        return
      }
      const createResp = await createSession(selectedAgent !== 'default' ? selectedAgent : undefined)
      if (createResp.success && createResp.data) {
        setSessionId(createResp.data.id)
        setSessionTitle('New Chat')
        setSessionAgent(createResp.data.agent_id || null)
        setMessages([])
      }
    }
    setLoading(false)
  }, [selectedAgent])

  useEffect(() => { load() }, [load])

  const newChat = useCallback(async () => {
    const resp = await createSession(selectedAgent !== 'default' ? selectedAgent : undefined)
    if (resp.success && resp.data) {
      setSessionId(resp.data.id)
      setSessionTitle('New Chat')
      setSessionAgent(resp.data.agent_id || null)
      setMessages([])
    }
  }, [selectedAgent])

  return { messages, setMessages, sessionId, setSessionId, sessionTitle, sessionAgent, loading, reload: load, newChat }
}
```

- [ ] **Step 5: Create ChatInput component**

`crates/claw/dashboard-ui/src/components/ChatInput.tsx`:
```tsx
import { useRef } from 'react'
import { Send } from 'lucide-react'
import styles from './ChatInput.module.css'

interface Props {
  onSend: (text: string) => void
  disabled: boolean
}

export default function ChatInput({ onSend, disabled }: Props) {
  const ref = useRef<HTMLTextAreaElement>(null)
  const [value, setValue] = useState('')

  const handleSend = () => {
    const text = value.trim()
    if (!text) return
    onSend(text)
    setValue('')
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSend()
    }
  }

  return (
    <div className={styles.area}>
      <div className={styles.container}>
        <textarea
          ref={ref}
          className={styles.input}
          value={value}
          onChange={e => setValue(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Type your message..."
          rows={1}
          disabled={disabled}
        />
        <button className={styles.sendBtn} onClick={handleSend} disabled={disabled || !value.trim()}>
          <Send size={15} /> Send
        </button>
      </div>
    </div>
  )
}
```

- [ ] **Step 6: Create StreamingBubble component**

`crates/claw/dashboard-ui/src/components/StreamingBubble.tsx`:
```tsx
import { Brain, Terminal } from 'lucide-react'
import MarkdownRenderer from './MarkdownRenderer'
import type { ToolCallMsg } from '../api'
import styles from './StreamingBubble.module.css'

interface Props {
  content: string
  reasoning: string
  toolCalls: ToolCallMsg[]
}

export default function StreamingBubble({ content, reasoning, toolCalls }: Props) {
  return (
    <div className={styles.bubble}>
      {reasoning && (
        <details className={styles.reasoning} open>
          <summary><Brain size={12} /> Thinking process</summary>
          <div className={styles.reasoningContent}>{reasoning}</div>
        </details>
      )}
      {content && (
        <div className={styles.content}>
          <MarkdownRenderer content={content} />
        </div>
      )}
      {toolCalls.length > 0 && (
        <details className={styles.toolCalls} open>
          <summary><Terminal size={12} /> Tool calls ({toolCalls.length})</summary>
          {/* ToolCallCard list */}
        </details>
      )}
      {!content && toolCalls.length === 0 && (
        <div className={styles.thinking}>
          <div className={styles.dots}><span/><span/><span/></div>
          Thinking...
        </div>
      )}
    </div>
  )
}
```

- [ ] **Step 7: Simplify Chat.tsx to use hooks**

`crates/claw/dashboard-ui/src/pages/Chat.tsx`:
```tsx
import { Plus, List, Bot, MessageSquare, Sparkles } from 'lucide-react'
import { useChatStream } from '../hooks/useChatStream'
import { useSessionLoader } from '../hooks/useSessionLoader'
import ChatInput from '../components/ChatInput'
import MessageList from '../components/MessageList'
import StreamingBubble from '../components/StreamingBubble'

interface Props {
  selectedAgent: string
  onNavigate?: (page: string) => void
  onSessionChange?: () => void
}

export default function ChatPage({ selectedAgent, onNavigate, onSessionChange }: Props) {
  const { messages, setMessages, sessionId, sessionTitle, sessionAgent, loading, reload, newChat } = useSessionLoader(selectedAgent)
  const { streaming, state, sendMessage, abort } = useChatStream()

  const handleSend = async (text: string) => {
    abort()
    setMessages(prev => [...prev, { role: 'user', content: text }])
    await sendMessage(text, selectedAgent !== 'default' ? selectedAgent : undefined)
    // After streaming completes, reload session to get persisted messages
    // State handling happens in useChatStream callbacks
  }

  return (
    <div className="chat-container">
      <div className="page-header">
        <div className="page-header-left">
          <MessageSquare size={16} />
          <h2>{sessionTitle}</h2>
          {(sessionAgent || selectedAgent) && (
            <span className="badge badge-info"><Bot size={10} />{sessionAgent || selectedAgent}</span>
          )}
        </div>
        <button className="btn btn-primary btn-sm" onClick={newChat} disabled={streaming}>
          <Plus size={14} /> New Chat
        </button>
      </div>

      <MessageList messages={messages} sessionId={sessionId} />

      {streaming && <StreamingBubble content={state.content} reasoning={state.reasoning} toolCalls={state.toolCalls} />}

      <ChatInput onSend={handleSend} disabled={streaming} />
    </div>
  )
}
```

- [ ] **Step 8: Create CSS Module files**

`crates/claw/dashboard-ui/src/components/ChatInput.module.css`:
```css
.area { padding: 16px 24px 20px; background: var(--bg-secondary); border-top: 1px solid var(--border); }
.container { display: flex; gap: 12px; align-items: flex-end; background: var(--bg-tertiary); border: 1px solid var(--border); border-radius: var(--radius-lg); padding: 8px 8px 8px 16px; }
.container:focus-within { border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-muted); }
.input { flex: 1; padding: 10px 0; border: none; background: transparent; color: var(--text-primary); font-size: 14px; font-family: inherit; resize: none; outline: none; }
.sendBtn { /* styles */ }
```

- [ ] **Step 9: Build and verify**

```bash
cd crates/claw/dashboard-ui && pnpm build
```

- [ ] **Step 10: Commit**

```bash
git add -A
git commit -m "refactor(dashboard-ui): add routing, split Chat.tsx, CSS Modules

- wouter routing replaces useState-based page switching
- useChatStream hook encapsulates SSE connection + incremental rendering
- useSessionLoader hook with single deserializeMessages function
- ChatInput, MessageList, StreamingBubble extracted as separate components
- CSS Modules for component-level style isolation
- New Chat endpoint uses single POST /api/chat (no separate stream step)"
```

---

### Task 8: Integration testing

**Files:**
- Run: `cargo test -p i-rs-claw-core -- --test-threads=1`
- Run: `cargo test -p i-rs-claw -- --test-threads=1`
- Run: `cargo check --workspace`

- [ ] **Step 1: Run all tests**

```bash
cargo test -p i-rs-claw-core -- --test-threads=1
cargo test -p i-rs-claw -- --test-threads=1
cargo test -p i-rs-api -- --test-threads=1
```

Expected: all existing tests pass. Fix any failures.

- [ ] **Step 2: Run clippy**

```bash
cargo clippy --workspace -- -D warnings
```

Fix any warnings.

- [ ] **Step 3: Run cargo fmt check**

```bash
cargo fmt --all --check
```

- [ ] **Step 4: Verify dashboard-ui builds**

```bash
cd crates/claw/dashboard-ui && pnpm build
```

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "test: verify all tests pass after dashboard redesign"
```

---

## Summary of File Changes

| Action | File |
|--------|------|
| CREATE | `crates/claw-core/Cargo.toml` |
| CREATE | `crates/claw-core/src/lib.rs` |
| MOVE | `crates/claw/src/{app,config,convstore,error,llm,mcp,memory,plugin,router,semantic,session,skill_store,tool_cache,utils,test_helpers}.rs` → `crates/claw-core/src/` |
| MOVE | `crates/claw/src/{core,providers,stats,storage,message,tools}/` → `crates/claw-core/src/` |
| MODIFY | `Cargo.toml` (workspace members) |
| MODIFY | `crates/claw/Cargo.toml` (add claw-core dep) |
| MODIFY | `crates/claw/src/main.rs` (CLI restructure, serve default) |
| CREATE | `crates/claw/src/lib.rs` |
| CREATE | `crates/claw/src/serve/mod.rs` |
| CREATE | `crates/claw/src/serve/routes.rs` (merged from dashboard/routes.rs) |
| MODIFY | `crates/claw-core/src/session.rs` (add user_id) |
| MODIFY | `crates/claw-core/src/stats/mod.rs` (add user_id) |
| MODIFY | `crates/claw-core/src/config.rs` (add DashboardUser) |
| MODIFY | `crates/claw-core/src/llm.rs` (enhance NewRound) |
| MODIFY | `crates/claw-core/src/core/engine/mod.rs` (incremental persist) |
| MODIFY | All storage backends (add user_id column/field) |
| REMOVE | `crates/claw/src/dashboard/mod.rs` (merged into serve/) |
| KEEP | `crates/claw/src/dashboard/assets.rs` (SPA serving) |
| KEEP | `crates/claw/src/dashboard-ui/` (React project, independent) |
| CREATE | `crates/claw/dashboard-ui/src/hooks/useChatStream.ts` |
| CREATE | `crates/claw/dashboard-ui/src/hooks/useSessionLoader.ts` |
| CREATE | `crates/claw/dashboard-ui/src/components/ChatInput.tsx` |
| CREATE | `crates/claw/dashboard-ui/src/components/MessageList.tsx` |
| CREATE | `crates/claw/dashboard-ui/src/components/StreamingBubble.tsx` |
| CREATE | `crates/claw/dashboard-ui/src/components/*.module.css` |
| MODIFY | `crates/claw/dashboard-ui/src/App.tsx` (wouter routing) |
| MODIFY | `crates/claw/dashboard-ui/src/pages/Chat.tsx` (simplified) |
| MODIFY | `crates/claw/dashboard-ui/src/api.ts` (single chat endpoint) |
