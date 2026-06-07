# Claw Refactor & Cleanup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Decompose 5 god modules (each >800 LOC) into focused sub-modules, delete or wire 7 entire dead-code modules (~2000 LOC unused), remove trivial shim files, externalize hardcoded data tables (pricing, themes), and consolidate the 4 chat endpoints down to 2.

**Architecture:** Each refactor is independently committable. Use `pub use` re-exports to preserve public API during migration. Tests run after each module split to catch breakage. Dead modules are deleted entirely (not gated) to prevent zombie code. Hardcoded tables move to TOML/JSON files loaded at startup.

**Tech Stack:** Rust, serde for TOML/JSON loading, `include_str!` macro optional.

**Estimated time:** 3–5 days (13 tasks, each independently committable).

---

## Scope — verified audit findings

### God modules to split

| File | LOC | Current mix | Target structure |
|------|-----|-------------|------------------|
| `crates/claw/src/app.rs` | 2010 | Re-exports + `InputState` + `App` state + scroll/render + cmd history + 500 LOC tests | `crates/claw/src/app/{mod,state,input,history,scroll,tests}.rs` |
| `crates/claw-core/src/core/mod.rs` | 1218 | `AppCore` + `AgentRuntime` + `AgentRuntimeStore` + storage init | `crates/claw-core/src/core/{mod,runtime,init}.rs` |
| `crates/claw-core/src/core/engine/mod.rs` | 1051 | `chat_loop` + `prepare_loop` + pipeline stages | `crates/claw-core/src/core/engine/{mod,loop,pipeline}.rs` |
| `crates/claw/src/ui/panels.rs` | 835 | Help + agent + skill + MCP + plugin + memory panels | `crates/claw/src/ui/panels/{mod,help,agent,skill,mcp,plugin,memory}.rs` |
| `crates/claw/src/ui/chat/components/tool_call.rs` | 848 | Streaming + status + args + result + error + collapse | Split by responsibility |
| `crates/claw/src/tui/handlers/key.rs` | 688 | Giant match on `KeyCode` | Dispatch table |
| `crates/claw/src/tui/handlers/overlay.rs` | 687 | Same shape | Same approach |

### Dead modules to delete or wire

| File | LOC | Status | Action |
|------|-----|--------|--------|
| `claw-core/src/core/checkpoint.rs` | 159 | Most `#[allow(dead_code)]` | Wire to `/api/checkpoints/restore` OR delete |
| `claw-core/src/core/context.rs` | 227 | All `#[allow(dead_code)]` | Delete (`ContextManager` unused; engine has its own compression) |
| `claw-core/src/core/layered_memory.rs` | 483 | Most methods `#[allow(dead_code)]` | Delete (`memory.rs` already provides `CrossSessionMemory`) |
| `claw-core/src/core/streaming.rs` | 147 | All `#[allow(dead_code)]` | Delete |
| `claw-core/src/core/tool_chain.rs` | 321 | All `#[allow(dead_code)]` | Delete |
| `claw-core/src/core/orchestration.rs` | 276 | Most methods `#[allow(dead_code)]` | Delete |
| `claw-core/src/core/planning.rs` | 372 | Mostly `#[allow(dead_code)]`, `parse_from_llm_output` used by `delegate` | Delete unused methods, move `parse_from_llm_output` into `delegate.rs` |

### Trivial shims to remove

- `crates/claw/src/lib.rs` (1 line)
- `crates/claw/src/config.rs` (1 line)
- `crates/claw/src/theme.rs` (1 line)

These force `use i_rs_claw_core::*` at call sites.

### Externalize data

| File | LOC | Content | Target |
|------|-----|---------|--------|
| `crates/claw-core/src/stats/pricing.rs` | 156 | Hardcoded model pricing | `crates/claw-core/data/pricing.toml` (loaded via serde at startup) |
| `crates/claw-core/src/theme.rs` (lines 58–379) | 320 | 20 `ThemePreset` static | `crates/claw-core/data/themes/*.json` (one file per theme) OR `themes.json` |

### Chat endpoint consolidation

| Endpoint | Status | Action |
|----------|--------|--------|
| `POST /api/chat` | Canonical | Keep |
| `POST /api/send_message` (`chat.rs:139`) | `#[allow(dead_code)]` | Delete |
| `GET /api/chat/stream` (`chat_stream`) | Backward compat | Keep with deprecation header |
| `GET /api/chat/stream/resume` (`chat_stream_resume`) | `_cursor` unused — fake resume | Either implement OR rename to `/replay` and document accordingly |

---

## Task 1: Audit shim usage

**Goal:** Produce a complete migration table of every consumer of the three 1-line shims before deleting them.

**Files:**
- Read: `crates/claw/src/lib.rs`
- Read: `crates/claw/src/config.rs`
- Read: `crates/claw/src/theme.rs`
- Search: `crates/claw/src/` for `use crate::config`, `use crate::theme`, `use crate::lib`, `use crate::App`, `use crate::InputState`, `use crate::Message`

- [ ] **Step 1: Read the three shim files**

```bash
# Confirm each is a single-line re-export
cat crates/claw/src/lib.rs
cat crates/claw/src/config.rs
cat crates/claw/src/theme.rs
```

Expected output (one line each, shape):
```rust
pub use i_rs_claw_core::config::*;
pub use i_rs_claw_core::theme::*;
pub use i_rs_claw_core::*;
```

- [ ] **Step 2: Grep for `use crate::config`**

Run:
```bash
rg -n 'use crate::config' crates/claw/src
```

Capture every hit with `file:line` into a markdown table.

- [ ] **Step 3: Grep for `use crate::theme`**

Run:
```bash
rg -n 'use crate::theme' crates/claw/src
```

- [ ] **Step 4: Grep for shim re-exported names**

Run:
```bash
rg -n 'use crate::\{App|InputState|Message|Config|ThemePreset' crates/claw/src
```

- [ ] **Step 5: Grep `lib.rs` / `main.rs` for `mod config` / `mod theme`**

Run:
```bash
rg -n '^mod (config|theme|lib)\b|^pub mod (config|theme|lib)\b' crates/claw/src
```

- [ ] **Step 6: Write the migration table**

Record the table in `docs/superpowers/plans/2026-06-07-claw-refactor-cleanup.md` under a new "Shim migration table" appendix section. The table columns are:

| File | Line | Current import | New import |
|------|------|----------------|------------|

For example:
| `crates/claw/src/main.rs` | 12 | `use crate::config::Config;` | `use i_rs_claw_core::config::Config;` |

- [ ] **Step 7: Commit**

```bash
git add docs/superpowers/plans/2026-06-07-claw-refactor-cleanup.md
git commit -m "docs(plan): add shim migration table for claw refactor"
```

---

## Task 2: Remove 1-line shims

**Goal:** Replace every shim consumer with `use i_rs_claw_core::*` directly, then delete the three shim files.

**Files:**
- Modify: every consumer identified in Task 1
- Delete: `crates/claw/src/lib.rs`
- Delete: `crates/claw/src/config.rs`
- Delete: `crates/claw/src/theme.rs`
- Modify: `crates/claw/src/main.rs` (remove `mod config;` / `mod theme;`)

- [ ] **Step 1: Apply migration table from Task 1**

For each row in the table, replace the import line with the direct `i_rs_claw_core::...` equivalent.

Example transformation:
```rust
// before
use crate::config::Config;
use crate::theme::current_theme;

// after
use i_rs_claw_core::config::Config;
use i_rs_claw_core::theme::current_theme;
```

- [ ] **Step 2: Remove `mod` declarations in `main.rs`**

In `crates/claw/src/main.rs`, delete any line of the form:
```rust
mod config;
mod theme;
```

Keep `mod app;`, `mod ui;`, etc.

- [ ] **Step 3: Delete the three shim files**

Run:
```bash
git rm crates/claw/src/lib.rs crates/claw/src/config.rs crates/claw/src/theme.rs
```

- [ ] **Step 4: Verify build**

Run:
```bash
cargo check -p i-rs-claw
cargo check -p i-rs-claw --features dashboard
```

Expected: 0 errors, 0 warnings.

- [ ] **Step 5: Run tests**

Run:
```bash
cargo test -p i-rs-claw --features dashboard -- --test-threads=1
```

Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "refactor(claw): remove 1-line shims, use i_rs_claw_core directly"
```

---

## Task 3: Split `app.rs` (2010 LOC)

**Goal:** Decompose `crates/claw/src/app.rs` into a directory with focused sub-modules. Public API is preserved via `pub use` re-exports in `app/mod.rs`.

**Target structure:**
```
crates/claw/src/app/
├── mod.rs       # public surface: pub use state::*, input::*, history::*, scroll::*; re-export App, InputState
├── state.rs     # struct App { ... } + impl App (state transitions, message log)
├── input.rs     # struct InputState + impl (cursor, insert, backspace, word movement)
├── history.rs   # CmdHistory struct + impl (ring buffer, navigation)
├── scroll.rs    # scroll/render helpers used by App
└── tests.rs     # all #[cfg(test)] unit tests moved here
```

**Files:**
- Create: `crates/claw/src/app/mod.rs`
- Create: `crates/claw/src/app/state.rs`
- Create: `crates/claw/src/app/input.rs`
- Create: `crates/claw/src/app/history.rs`
- Create: `crates/claw/src/app/scroll.rs`
- Create: `crates/claw/src/app/tests.rs`
- Delete: `crates/claw/src/app.rs`

- [ ] **Step 1: Read existing `app.rs` end-to-end**

Run:
```bash
wc -l crates/claw/src/app.rs
```
Skim each section, note which lines belong to: `InputState`, `App`, scroll helpers, history, tests.

- [ ] **Step 2: Create `app/` directory**

Run:
```bash
mkdir -p crates/claw/src/app
git mv crates/claw/src/app.rs crates/claw/src/app/_legacy.rs
```

(Keep `_legacy.rs` temporarily as a reference; delete in Step 9.)

- [ ] **Step 3: Write `app/input.rs`**

Move `InputState` struct + impl block. Include all cursor/insert/delete/word-movement methods.

```rust
// crates/claw/src/app/input.rs
use ratatui::widgets::{Block, Borders, Paragraph};

#[derive(Debug, Clone)]
pub struct InputState {
    pub value: String,
    pub cursor: usize,
    // ...
}

impl InputState {
    pub fn new() -> Self { /* ... */ }
    pub fn insert_char(&mut self, c: char) { /* ... */ }
    pub fn backspace(&mut self) { /* ... */ }
    pub fn delete_word_back(&mut self) { /* ... */ }
    pub fn move_word_left(&mut self) { /* ... */ }
    pub fn move_word_right(&mut self) { /* ... */ }
    // ... all input methods
}

impl Default for InputState {
    fn default() -> Self { Self::new() }
}
```

- [ ] **Step 4: Write `app/history.rs`**

Move command history ring buffer.

```rust
// crates/claw/src/app/history.rs
const HISTORY_CAP: usize = 256;

#[derive(Debug, Clone)]
pub struct CmdHistory {
    buf: Vec<String>,
    head: usize,
    cursor: Option<usize>,
}

impl CmdHistory {
    pub fn new() -> Self { /* ... */ }
    pub fn push(&mut self, line: impl Into<String>) { /* ... */ }
    pub fn prev(&mut self) -> Option<&str> { /* ... */ }
    pub fn next(&mut self) -> Option<&str> { /* ... */ }
    pub fn reset_cursor(&mut self) { /* ... */ }
}
```

- [ ] **Step 5: Write `app/scroll.rs`**

Move scroll/render helpers (e.g. `scroll_to_bottom`, `scroll_up`, `scroll_down`, anything that manipulates `ScrollOffset`).

```rust
// crates/claw/src/app/scroll.rs
use crate::app::state::App;

impl App {
    pub fn scroll_to_bottom(&mut self) { /* ... */ }
    pub fn scroll_up(&mut self, n: usize) { /* ... */ }
    pub fn scroll_down(&mut self, n: usize) { /* ... */ }
    pub fn current_scroll_offset(&self) -> usize { /* ... */ }
}
```

(Use `impl App` in a sibling file — Rust allows multiple `impl` blocks across modules for the same type as long as the type is visible.)

- [ ] **Step 6: Write `app/state.rs`**

Move `App` struct definition + core state methods (constructor, message log, mode transitions, overlay state).

```rust
// crates/claw/src/app/state.rs
use crate::app::input::InputState;
use crate::app::history::CmdHistory;
use i_rs_claw_core::app::Message;

#[derive(Debug)]
pub struct App {
    pub input: InputState,
    pub history: CmdHistory,
    pub messages: Vec<Message>,
    pub scroll_offset: usize,
    // ...
}

impl App {
    pub fn new() -> Self { /* ... */ }
    pub fn push_message(&mut self, m: Message) { /* ... */ }
    // core methods only — scroll/input helpers live in their own files
}
```

- [ ] **Step 7: Write `app/tests.rs`**

Move every `#[cfg(test)]` block from `_legacy.rs` here. Use:
```rust
// crates/claw/src/app/tests.rs
use super::*;

#[test]
fn input_state_insert_char_advances_cursor() { /* ... */ }
// ... all tests
```

- [ ] **Step 8: Write `app/mod.rs` re-export surface**

```rust
// crates/claw/src/app/mod.rs
mod state;
mod input;
mod history;
mod scroll;

#[cfg(test)]
mod tests;

pub use state::App;
pub use input::InputState;
pub use history::CmdHistory;
```

- [ ] **Step 9: Delete `_legacy.rs`**

Run:
```bash
git rm crates/claw/src/app/_legacy.rs
```

- [ ] **Step 10: Verify build**

Run:
```bash
cargo check -p i-rs-claw --features dashboard
```
Expected: 0 errors.

- [ ] **Step 11: Run tests**

Run:
```bash
cargo test -p i-rs-claw --features dashboard -- --test-threads=1
```
Expected: all tests pass (the tests in `app/tests.rs` count toward `i-rs-claw` test count).

- [ ] **Step 12: Commit**

```bash
git add -A
git commit -m "refactor(claw): split app.rs (2010 LOC) into app/{state,input,history,scroll,tests}.rs"
```

---

## Task 4: Split `claw-core/src/core/mod.rs` (1218 LOC)

**Goal:** Decompose the god module that holds `AppCore`, `AgentRuntime`, and `AgentRuntimeStore`.

**Target structure:**
```
crates/claw-core/src/core/
├── mod.rs       # AppCore only + module declarations + pub use re-exports
├── runtime.rs   # AgentRuntime + AgentRuntimeStore
└── init.rs      # storage initialization helpers (build_storage, init_storage_backends, etc.)
```

**Files:**
- Modify: `crates/claw-core/src/core/mod.rs` (keep only `AppCore`)
- Create: `crates/claw-core/src/core/runtime.rs`
- Create: `crates/claw-core/src/core/init.rs`

- [ ] **Step 1: Read existing `core/mod.rs`**

Run:
```bash
wc -l crates/claw-core/src/core/mod.rs
```
Note which line ranges contain: `AppCore` struct + impl, `AgentRuntime`, `AgentRuntimeStore`, storage init helpers.

- [ ] **Step 2: Write `core/runtime.rs`**

```rust
// crates/claw-core/src/core/runtime.rs
use crate::config::AgentConfig;
use crate::llm::LlmEvent;
use tokio::sync::mpsc;

pub struct AgentRuntime {
    pub config: AgentConfig,
    // ... fields moved from mod.rs
}

impl AgentRuntime {
    // ... methods moved from mod.rs
}

#[derive(Default)]
pub struct AgentRuntimeStore {
    // ... fields
}

impl AgentRuntimeStore {
    // ... methods
}
```

- [ ] **Step 3: Write `core/init.rs`**

```rust
// crates/claw-core/src/core/init.rs
use crate::storage::StorageBackend;

pub fn build_storage(cfg: &crate::config::Config) -> StorageBackend {
    // ... moved from mod.rs
}

pub fn init_storage_backends(core: &mut super::AppCore) {
    // ... moved from mod.rs
}
```

- [ ] **Step 4: Trim `core/mod.rs`**

Keep only:
```rust
// crates/claw-core/src/core/mod.rs
mod runtime;
mod init;
// ... other existing submodules (engine, checkpoint, etc.)

pub use runtime::{AgentRuntime, AgentRuntimeStore};
pub use init::{build_storage, init_storage_backends};

use crate::config::Config;
use crate::session::SessionManager;

pub struct AppCore {
    // ... fields
}

impl AppCore {
    pub fn new(cfg: Config) -> anyhow::Result<Self> {
        // ... uses init::build_storage, init::init_storage_backends
    }
    // ... other AppCore-only methods
}
```

- [ ] **Step 5: Verify build**

Run:
```bash
cargo check -p i-rs-claw-core
cargo check --workspace
```
Expected: 0 errors.

- [ ] **Step 6: Run tests**

Run:
```bash
cargo test -p i-rs-claw-core
cargo test -p i-rs-claw --features dashboard -- --test-threads=1
```
Expected: all tests pass.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "refactor(claw-core): split core/mod.rs into core/{runtime,init}.rs"
```

---

## Task 5: Split `core/engine/mod.rs` (1051 LOC)

**Goal:** Decompose the chat-loop engine into `loop.rs` and `pipeline.rs`.

**Target structure:**
```
crates/claw-core/src/core/engine/
├── mod.rs        // module declarations + pub use + builder (if small) or move to builder.rs
├── loop.rs       // chat_loop + prepare_loop (the main async loop)
├── pipeline.rs   // pipeline stages: gather_tool_calls, execute_tools, format_results, etc.
└── builder.rs    // (existing — keep) builder/constructor logic
```

**Files:**
- Modify: `crates/claw-core/src/core/engine/mod.rs`
- Create: `crates/claw-core/src/core/engine/loop.rs`
- Create: `crates/claw-core/src/core/engine/pipeline.rs`

- [ ] **Step 1: Read existing `engine/mod.rs`**

Run:
```bash
wc -l crates/claw-core/src/core/engine/mod.rs
ls crates/claw-core/src/core/engine/
```

- [ ] **Step 2: Write `engine/loop.rs`**

Move `chat_loop` and `prepare_loop` here. Use `super::pipeline::*` for stage helpers.

```rust
// crates/claw-core/src/core/engine/loop.rs
use super::pipeline::{gather_tool_calls, execute_tools, format_results};
use crate::llm::LlmEvent;

pub async fn chat_loop(/* args */) -> anyhow::Result<()> {
    // ... moved from mod.rs
}

fn prepare_loop(/* args */) -> LoopState {
    // ... moved from mod.rs
}
```

- [ ] **Step 3: Write `engine/pipeline.rs`**

Move every pipeline-stage helper here:

```rust
// crates/claw-core/src/core/engine/pipeline.rs
use crate::llm::LlmEvent;

pub(super) fn gather_tool_calls(events: &[LlmEvent]) -> Vec<ToolCall> { /* ... */ }
pub(super) async fn execute_tools(calls: &[ToolCall]) -> Vec<ToolResult> { /* ... */ }
pub(super) fn format_results(results: &[ToolResult]) -> Vec<Value> { /* ... */ }
```

Mark helpers `pub(super)` so only `loop.rs` and `mod.rs` see them.

- [ ] **Step 4: Trim `engine/mod.rs`**

```rust
// crates/claw-core/src/core/engine/mod.rs
mod loop_;
mod pipeline;
mod builder; // if present

pub use loop_::chat_loop;
pub use builder::*; // keep existing exports
```

> **Note:** `loop` is a Rust keyword; use `loop_` as the module name and `pub use loop_ as lo` / `pub use loop_::chat_loop` to keep the public surface stable.

- [ ] **Step 5: Verify build**

Run:
```bash
cargo check -p i-rs-claw-core
cargo check --workspace
```

- [ ] **Step 6: Run tests**

Run:
```bash
cargo test -p i-rs-claw-core
cargo test -p i-rs-claw --features dashboard -- --test-threads=1
```

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "refactor(claw-core): split engine/mod.rs into engine/{loop_,pipeline}.rs"
```

---

## Task 6: Split `ui/panels.rs` (835 LOC)

**Goal:** Decompose `panels.rs` into one file per panel kind, all rendering from a shared `PanelCtx`.

**Target structure:**
```
crates/claw/src/ui/panels/
├── mod.rs     // pub fn render_panel(frame, area, app, kind); module decls
├── common.rs  // struct PanelCtx<'a> { app: &'a App, theme: &'a Theme, area: Rect }
├── help.rs    // render_help_panel(ctx)
├── agent.rs   // render_agent_panel(ctx)
├── skill.rs   // render_skill_panel(ctx)
├── mcp.rs     // render_mcp_panel(ctx)
├── plugin.rs  // render_plugin_panel(ctx)
└── memory.rs  // render_memory_panel(ctx)
```

**Files:**
- Create: `crates/claw/src/ui/panels/mod.rs`
- Create: `crates/claw/src/ui/panels/common.rs`
- Create: `crates/claw/src/ui/panels/help.rs`
- Create: `crates/claw/src/ui/panels/agent.rs`
- Create: `crates/claw/src/ui/panels/skill.rs`
- Create: `crates/claw/src/ui/panels/mcp.rs`
- Create: `crates/claw/src/ui/panels/plugin.rs`
- Create: `crates/claw/src/ui/panels/memory.rs`
- Delete: `crates/claw/src/ui/panels.rs`

- [ ] **Step 1: Read existing `panels.rs`**

Run:
```bash
wc -l crates/claw/src/ui/panels.rs
```
For each `pub fn render_*_panel`, note line range.

- [ ] **Step 2: Create directory + move legacy**

Run:
```bash
mkdir -p crates/claw/src/ui/panels
git mv crates/claw/src/ui/panels.rs crates/claw/src/ui/panels/_legacy.rs
```

- [ ] **Step 3: Write `panels/common.rs`**

```rust
// crates/claw/src/ui/panels/common.rs
use crate::app::App;
use i_rs_claw_core::theme::Theme;
use ratatui::layout::Rect;

pub struct PanelCtx<'a> {
    pub app: &'a App,
    pub theme: &'a Theme,
    pub area: Rect,
}
```

- [ ] **Step 4: Write one file per panel**

For each `render_X_panel` function in `_legacy.rs`:

```rust
// crates/claw/src/ui/panels/help.rs
use super::common::PanelCtx;

pub fn render_help_panel(ctx: PanelCtx<'_>) {
    // ... body moved from _legacy.rs
}
```

Repeat for `agent.rs`, `skill.rs`, `mcp.rs`, `plugin.rs`, `memory.rs`.

- [ ] **Step 5: Write `panels/mod.rs`**

```rust
// crates/claw/src/ui/panels/mod.rs
mod common;
mod help;
mod agent;
mod skill;
mod mcp;
mod plugin;
mod memory;

pub use help::render_help_panel;
pub use agent::render_agent_panel;
pub use skill::render_skill_panel;
pub use mcp::render_mcp_panel;
pub use plugin::render_plugin_panel;
pub use memory::render_memory_panel;

pub use common::PanelCtx;
```

- [ ] **Step 6: Update `ui/mod.rs`**

Find any `mod panels;` declaration in `crates/claw/src/ui/mod.rs`. Since `panels/` is now a directory, the existing `mod panels;` line works unchanged. Verify no other references.

Run:
```bash
rg -n 'panels::' crates/claw/src
```
Confirm all `panels::render_X_panel` calls still resolve.

- [ ] **Step 7: Delete `_legacy.rs`**

Run:
```bash
git rm crates/claw/src/ui/panels/_legacy.rs
```

- [ ] **Step 8: Verify build + tests**

Run:
```bash
cargo check -p i-rs-claw --features dashboard
cargo test -p i-rs-claw --features dashboard -- --test-threads=1
```

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "refactor(claw): split ui/panels.rs into one file per panel kind"
```

---

## Task 7: Split `ui/chat/components/tool_call.rs` (848 LOC)

**Goal:** Decompose by responsibility — streaming state, args rendering, result rendering, error rendering, collapse/expand state.

**Target structure:**
```
crates/claw/src/ui/chat/components/tool_call/
├── mod.rs       // pub use re-exports, ToolCallView entry point
├── status.rs    // status badge / spinner
├── args.rs      // render_tool_call_args (JSON pretty-print, collapse)
├── result.rs    // render_tool_call_result (truncate, scroll)
├── error.rs     // render_tool_call_error (red highlight)
└── streaming.rs // streaming-state tracking
```

**Files:**
- Create all files in `crates/claw/src/ui/chat/components/tool_call/`
- Delete: `crates/claw/src/ui/chat/components/tool_call.rs`

- [ ] **Step 1: Read existing `tool_call.rs`**

Run:
```bash
wc -l crates/claw/src/ui/chat/components/tool_call.rs
```
Note line ranges for each function.

- [ ] **Step 2: Create directory + move legacy**

Run:
```bash
mkdir -p crates/claw/src/ui/chat/components/tool_call
git mv crates/claw/src/ui/chat/components/tool_call.rs \
       crates/claw/src/ui/chat/components/tool_call/_legacy.rs
```

- [ ] **Step 3: Write each sub-module**

For each responsibility:
```rust
// crates/claw/src/ui/chat/components/tool_call/args.rs
use ratatui::text::Text;

pub fn render_args(args: &serde_json::Value, collapsed: bool) -> Text<'static> {
    // ... moved from _legacy.rs
}
```

Repeat for `status.rs`, `result.rs`, `error.rs`, `streaming.rs`.

- [ ] **Step 4: Write `tool_call/mod.rs`**

```rust
// crates/claw/src/ui/chat/components/tool_call/mod.rs
mod status;
mod args;
mod result;
mod error;
mod streaming;

pub use status::render_status;
pub use args::render_args;
pub use result::render_result;
pub use error::render_error;

// Top-level entry point kept here
use ratatui::text::Text;

pub fn render_tool_call(/* args */) -> Text<'static> {
    // ... composes the above
}
```

- [ ] **Step 5: Delete `_legacy.rs`**

Run:
```bash
git rm crates/claw/src/ui/chat/components/tool_call/_legacy.rs
```

- [ ] **Step 6: Verify build + tests**

Run:
```bash
cargo check -p i-rs-claw --features dashboard
cargo test -p i-rs-claw --features dashboard -- --test-threads=1
```

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "refactor(claw): split tool_call.rs (848 LOC) by responsibility"
```

---

## Task 8: Delete dead modules

**Goal:** Remove 6 entirely-dead modules from `claw-core/src/core/`. Each module is mostly/all `#[allow(dead_code)]`.

**Files to delete:**
- `crates/claw-core/src/core/checkpoint.rs` (159 LOC) — *delete, see Task 12 note*
- `crates/claw-core/src/core/context.rs` (227 LOC)
- `crates/claw-core/src/core/layered_memory.rs` (483 LOC)
- `crates/claw-core/src/core/streaming.rs` (147 LOC)
- `crates/claw-core/src/core/tool_chain.rs` (321 LOC)
- `crates/claw-core/src/core/orchestration.rs` (276 LOC)

- [ ] **Step 1: Grep for any external reference to each module**

Run for each module:
```bash
for mod in checkpoint context layered_memory streaming tool_chain orchestration; do
  echo "=== $mod ==="
  rg -n "${mod}::|use (crate|super)::${mod}|use i_rs_claw_core::core::${mod}" \
    crates/claw crates/claw-core crates/cli-api crates/mcp
done
```

Expected output: only self-references inside the module file itself, plus the `mod` declaration in `core/mod.rs`. Any other hit is a real caller — must be addressed before deletion.

- [ ] **Step 2: Grep for uses of exported types**

Run:
```bash
rg -n 'ContextManager|LayeredMemory|StreamingState|ToolChain|Orchestrator|Checkpoint' \
  crates/claw crates/claw-core crates/cli-api crates/mcp
```

- [ ] **Step 3: Handle any unexpected callers found in Steps 1–2**

For each caller that is NOT inside the dead module itself:
- If the call site is itself dead code (e.g. another `#[allow(dead_code)]` function), delete the call site too.
- If the call site is live, **STOP** and re-evaluate: either keep the module or refactor the caller. Document the decision in this plan's appendix.

- [ ] **Step 4: Remove `mod` declarations from `crates/claw-core/src/core/mod.rs`**

Delete lines like:
```rust
mod checkpoint;
mod context;
mod layered_memory;
mod streaming;
mod tool_chain;
mod orchestration;
```

Also delete any corresponding `pub use` lines.

- [ ] **Step 5: Delete the files**

Run:
```bash
git rm crates/claw-core/src/core/checkpoint.rs \
       crates/claw-core/src/core/context.rs \
       crates/claw-core/src/core/layered_memory.rs \
       crates/claw-core/src/core/streaming.rs \
       crates/claw-core/src/core/tool_chain.rs \
       crates/claw-core/src/core/orchestration.rs
```

- [ ] **Step 6: Verify build**

Run:
```bash
cargo check -p i-rs-claw-core
cargo check --workspace
```
Expected: 0 errors. If errors mention the deleted types, return to Step 3.

- [ ] **Step 7: Run tests**

Run:
```bash
cargo test --workspace -- --test-threads=1
```

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "refactor(claw-core): delete 6 dead modules (~1600 LOC)"
```

---

## Task 9: Trim `planning.rs` (372 LOC)

**Goal:** Move the one used function (`parse_from_llm_output`) into `delegate.rs` as a private helper, then delete `planning.rs`.

**Files:**
- Read: `crates/claw-core/src/core/planning.rs`
- Read: `crates/claw-core/src/core/delegate.rs` (or wherever the caller lives)
- Modify: `crates/claw-core/src/core/delegate.rs`
- Delete: `crates/claw-core/src/core/planning.rs`
- Modify: `crates/claw-core/src/core/mod.rs`

- [ ] **Step 1: Locate the live caller**

Run:
```bash
rg -n 'parse_from_llm_output\|planning::' crates/claw-core/src
```
Confirm the only live caller is in `delegate.rs`.

- [ ] **Step 2: Copy `parse_from_llm_output` into `delegate.rs`**

Add as a private `fn` near the top of `delegate.rs`:
```rust
// crates/claw-core/src/core/delegate.rs
fn parse_from_llm_output(text: &str) -> Vec<PlanStep> {
    // ... exact body from planning.rs
}
```

If `PlanStep` is defined in `planning.rs`, also move it (or change the return type to whatever the caller actually needs — likely `Vec<String>` or `Vec<Value>`).

- [ ] **Step 3: Replace `planning::parse_from_llm_output` calls**

In `delegate.rs`, replace:
```rust
let steps = super::planning::parse_from_llm_output(&output);
// becomes
let steps = parse_from_llm_output(&output);
```

- [ ] **Step 4: Remove `mod planning` declaration**

In `crates/claw-core/src/core/mod.rs`, delete:
```rust
mod planning;
// and any pub use planning::*
```

- [ ] **Step 5: Delete `planning.rs`**

Run:
```bash
git rm crates/claw-core/src/core/planning.rs
```

- [ ] **Step 6: Verify build + tests**

Run:
```bash
cargo check --workspace
cargo test -p i-rs-claw-core
```

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "refactor(claw-core): inline parse_from_llm_output into delegate.rs, delete planning.rs"
```

---

## Task 10: Externalize `pricing.toml`

**Goal:** Move the hardcoded model pricing table from `stats/pricing.rs` to `crates/claw-core/data/pricing.toml`, loaded at startup via `include_str!` + `toml::from_str`.

**Files:**
- Create: `crates/claw-core/data/pricing.toml`
- Modify: `crates/claw-core/src/stats/pricing.rs`
- Modify: `crates/claw-core/Cargo.toml` (ensure `toml` dep present, `data/` included in package)

- [ ] **Step 1: Read existing `pricing.rs`**

Run:
```bash
cat crates/claw-core/src/stats/pricing.rs
```
Capture every `(provider, model) → (input_price, output_price)` entry.

- [ ] **Step 2: Write `data/pricing.toml`**

```toml
# crates/claw-core/data/pricing.toml
# Prices in USD per 1M tokens.

[[models]]
provider = "openai"
model = "gpt-4o"
input_per_mtok = 5.0
output_per_mtok = 15.0

[[models]]
provider = "anthropic"
model = "claude-3-5-sonnet"
input_per_mtok = 3.0
output_per_mtok = 15.0

# ... one entry per model in the current static table
```

- [ ] **Step 3: Replace `pricing.rs` with loader**

```rust
// crates/claw-core/src/stats/pricing.rs
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct PricingEntry {
    provider: String,
    model: String,
    input_per_mtok: f64,
    output_per_mtok: f64,
}

#[derive(Debug, Deserialize)]
struct PricingFile {
    models: Vec<PricingEntry>,
}

pub fn load_pricing() -> HashMap<(String, String), (f64, f64)> {
    let text = include_str!("../../data/pricing.toml");
    let file: PricingFile = toml::from_str(text).expect("pricing.toml is valid");
    file.models
        .into_iter()
        .map(|e| ((e.provider, e.model), (e.input_per_mtok, e.output_per_mtok)))
        .collect()
}
```

- [ ] **Step 4: Update consumers**

Find every caller of the old static table:
```bash
rg -n 'PRICING\|pricing::\|get_pricing' crates/claw-core/src crates/claw/src
```

Replace with a single `once_cell::sync::Lazy` (or `std::sync::OnceLock`) that calls `load_pricing()`:

```rust
use std::sync::OnceLock;

static PRICING: OnceLock<HashMap<(String, String), (f64, f64)>> = OnceLock::new();

fn pricing() -> &'static HashMap<(String, String), (f64, f64)> {
    PRICING.get_or_init(load_pricing)
}
```

- [ ] **Step 5: Add `toml` workspace dep if missing**

Check root `Cargo.toml`:
```bash
rg -n '^toml\b' Cargo.toml
```
If absent, add `toml = "0.8"` to `[workspace.dependencies]`, then add `toml.workspace = true` to `crates/claw-core/Cargo.toml`.

- [ ] **Step 6: Include `data/` in package**

In `crates/claw-core/Cargo.toml`:
```toml
[package]
include = ["src/**", "data/**", "Cargo.toml"]
```

`include_str!` is a compile-time macro, so this is for `cargo publish` to ship the data file.

- [ ] **Step 7: Verify build + tests**

Run:
```bash
cargo check -p i-rs-claw-core
cargo test -p i-rs-claw-core
```

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "refactor(claw-core): externalize model pricing to data/pricing.toml"
```

---

## Task 11: Externalize themes

**Goal:** Move the 20 `ThemePreset` statics from `theme.rs` (lines 58–379) to JSON files loaded at startup.

**Files:**
- Create: `crates/claw-core/data/themes/` (one JSON per theme)
- Modify: `crates/claw-core/src/theme.rs`

- [ ] **Step 1: Read existing theme table**

Run:
```bash
sed -n '58,379p' crates/claw-core/src/theme.rs
```
List every `ThemePreset { ... }` block.

- [ ] **Step 2: Write one JSON file per theme**

For each preset, create e.g. `crates/claw-core/data/themes/dark.json`:

```json
{
  "name": "dark",
  "fg": "#dddddd",
  "bg": "#1c1c1c",
  "accent": "#7aa2f7",
  "muted": "#888888",
  "border": "#444444",
  "error": "#f7768e",
  "warning": "#e0af68",
  "success": "#9ece6a",
  "user_msg": "#7aa2f7",
  "assistant_msg": "#73daca"
}
```

Repeat for all 20 presets. (Field names must match `ThemePreset`'s serde schema — adapt accordingly.)

- [ ] **Step 3: Replace static theme table with loader**

```rust
// crates/claw-core/src/theme.rs (rewritten)
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Deserialize, Clone)]
pub struct ThemePreset {
    pub name: String,
    pub fg: String,
    pub bg: String,
    pub accent: String,
    pub muted: String,
    pub border: String,
    pub error: String,
    pub warning: String,
    pub success: String,
    pub user_msg: String,
    pub assistant_msg: String,
}

static THEMES: OnceLock<HashMap<String, ThemePreset>> = OnceLock::new();

pub fn themes() -> &'static HashMap<String, ThemePreset> {
    THEMES.get_or_init(load_themes)
}

fn load_themes() -> HashMap<String, ThemePreset> {
    let entries: &[(&str, &str)] = &[
        ("dark", include_str!("../data/themes/dark.json")),
        ("light", include_str!("../data/themes/light.json")),
        // ... one entry per theme file
    ];
    entries
        .iter()
        .map(|(name, text)| {
            let preset: ThemePreset =
                serde_json::from_str(text).expect(&format!("invalid theme JSON for {}", name));
            (preset.name.clone(), preset)
        })
        .collect()
}
```

> **Alternative:** use a single `themes.json` with an array. Pick whichever is more ergonomic; the file-per-theme version makes git diffs cleaner when adjusting a single theme.

- [ ] **Step 4: Update consumers**

Find all callers of `ThemePreset::dark` / `static DARK: ThemePreset = ...` etc:
```bash
rg -n 'ThemePreset::|THEMES\.\|static [A-Z_]+: ThemePreset' crates/claw-core/src crates/claw/src
```
Replace with `themes().get("dark").cloned().unwrap_or_default()`.

- [ ] **Step 5: Verify build + tests**

Run:
```bash
cargo check --workspace
cargo test -p i-rs-claw-core
cargo test -p i-rs-claw --features dashboard -- --test-threads=1
```

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "refactor(claw-core): externalize 20 theme presets to data/themes/*.json"
```

---

## Task 12: Consolidate chat endpoints

**Goal:** Delete the dead `send_message` endpoint and decide the fate of `chat_stream_resume`.

**Files:**
- Modify: `crates/claw/src/server/routes/chat.rs` (or `crates/claw/src/dashboard/routes.rs` — find via Grep)
- Modify: route registration in the router builder

- [ ] **Step 1: Locate the 4 endpoints**

Run:
```bash
rg -n 'fn chat\b|fn send_message|fn chat_stream\b|fn chat_stream_resume' \
  crates/claw/src
```
Capture exact file:line for each.

- [ ] **Step 2: Read `chat_stream_resume`**

Run:
```bash
rg -n -A 30 'fn chat_stream_resume' crates/claw/src
```
Confirm whether `_cursor` is truly unused and whether the function actually resumes from a stored state, or just replays.

- [ ] **Step 3: Delete `send_message`**

Remove the handler function and any `Router::new().route("/api/send_message", ...)` registration line.

```bash
# Example expected diff
- async fn send_message(...) -> impl IntoResponse { ... }
- let app = Router::new()
-     .route("/api/send_message", post(send_message))
```

- [ ] **Step 4: Decide on `chat_stream_resume`**

Pick ONE:
- **(a) Implement properly:** store cursors server-side, accept `?since=<cursor>`, return only new events.
- **(b) Rename to `/replay`:** document that it re-emits the full stream from session start, no resume semantics.

Default choice if no product requirement is given: **(b)**, since this plan is a cleanup, not a feature add.

If (b):
```rust
#[get("/api/chat/replay")]
async fn chat_replay(/* same args */) -> impl IntoResponse { /* same body */ }
```
Update route registration, update dashboard clients.

- [ ] **Step 5: Add deprecation header to `chat_stream` (optional)**

In the handler:
```rust
response.headers_mut().insert(
    "Deprecation",
    HeaderValue::from_static("true"),
);
```
Keep for backward compatibility.

- [ ] **Step 6: Update dashboard clients if route renamed**

Run:
```bash
rg -n 'chat_stream_resume\|/api/chat/stream/resume' \
  crates/claw/dashboard-ui/src apps/IrsClawApp apps/IrsClawMiniProgram
```
Replace with `/api/chat/replay` where applicable.

- [ ] **Step 7: Verify build + tests**

Run:
```bash
cargo check -p i-rs-claw --features dashboard
cargo test -p i-rs-claw --features dashboard -- --test-threads=1
```

If dashboard-ui is affected:
```bash
cd crates/claw/dashboard-ui && pnpm build
```

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "refactor(claw): delete dead send_message endpoint, rename chat_stream_resume to /replay"
```

---

## Task 13: Final verification + commit

**Goal:** Confirm the entire workspace is green after all refactors.

- [ ] **Step 1: Format check**

Run:
```bash
cargo fmt --all --check
```
If it fails, run `cargo fmt --all` and commit the formatting.

- [ ] **Step 2: Workspace check**

Run:
```bash
cargo check --workspace
```
Expected: 0 errors, 0 warnings.

- [ ] **Step 3: Clippy**

Run:
```bash
cargo clippy --workspace -- -D warnings
```
Expected: 0 warnings.

- [ ] **Step 4: Full test suite**

Run:
```bash
cargo test --workspace -- --test-threads=1
```
Expected: all tests pass.

If the 15 known tokio-runtime-nesting dashboard tests still fail, document them as pre-existing in the commit message — they are not in scope for this refactor.

- [ ] **Step 5: Dashboard UI build**

Run:
```bash
cd crates/claw/dashboard-ui && pnpm build
```
Expected: success.

- [ ] **Step 6: cargo-deny check**

Run:
```bash
cargo deny check 2>/dev/null || true
```
Confirm no new license/security issues introduced.

- [ ] **Step 7: LOC summary**

Run:
```bash
git diff --stat main...HEAD -- crates/claw crates/claw-core
```
Capture the total LOC delta in the final commit message.

- [ ] **Step 8: Final commit (if any cleanup)**

If formatting or any minor fix was needed:
```bash
git add -A
git commit -m "chore: final cleanup after claw refactor"
```

---

## Verification

After all 13 tasks:

1. **`cargo check --workspace`** — 0 errors, 0 warnings.
2. **`cargo clippy --workspace -- -D warnings`** — 0 warnings.
3. **`cargo fmt --all --check`** — clean.
4. **`cargo test --workspace -- --test-threads=1`** — all tests pass (except documented pre-existing tokio failures).
5. **`pnpm build`** in `crates/claw/dashboard-ui` — success.
6. **No `#[allow(dead_code)]`** in `craw-core/src/core/{checkpoint,context,layered_memory,streaming,tool_chain,orchestration,planning}.rs` — these files should no longer exist.
7. **No shim references** — `rg 'use crate::config|use crate::theme' crates/claw/src` returns 0 hits.
8. **No `send_message` route** — `rg '/api/send_message' crates/claw/src` returns 0 hits.
9. **LOC reduction** — overall diff should show ~2000 LOC deleted from `claw-core`, ~1500 LOC redistributed in `claw` (net delta ~−2000 to −3000 LOC across the two crates).

---

## Appendix A: Shim migration table

(Filled in during Task 1, Step 6.)

| File | Line | Current import | New import |
|------|------|----------------|------------|
| _to be filled_ | | | |

---

## Appendix B: Notes on `tui/handlers/key.rs` and `overlay.rs`

These two 600+ LOC handler files use a giant `match` on `KeyCode`. They are NOT included as required tasks in this plan because their structure, while verbose, is sequential and easy to navigate.

If a follow-up cleanup is desired, the recommended approach is a dispatch table:

```rust
// Before
match key.code {
    KeyCode::Char('a') => { /* 30 lines */ }
    KeyCode::Char('b') => { /* 30 lines */ }
    // ... 20 more arms
}

// After
let handlers: &[(&dyn Fn(&mut App), KeyCode)] = &[
    (&handle_a, KeyCode::Char('a')),
    (&handle_b, KeyCode::Char('b')),
];
for (handler, code) in handlers {
    if key.code == *code { handler(app); return; }
}
```

Defer to a separate plan if measurable benefit is needed.

---

## Appendix C: Task execution order

Tasks 1–2 must run first (shim removal enables everything else to use direct imports).

After Task 2:
- **Parallel-safe:** Tasks 3, 4, 5, 6, 7, 10, 11 (touch disjoint files).
- **Sequential:** Task 8 depends on no live callers — verify with Grep before deletion.
- **Sequential:** Task 9 depends on Task 8 (or at minimum on the Grep in Task 8 Step 1).
- **Last:** Task 12 (chat endpoint consolidation) — touches routes only.
- **Last:** Task 13 (final verification).

Recommended commit order:
1. Task 1 (audit)
2. Task 2 (shim removal)
3. Tasks 3 → 4 → 5 (claw-core splits) → 6 → 7 (claw splits)
4. Task 8 (dead module deletion) → Task 9 (planning trim)
5. Task 10 (pricing) → Task 11 (themes)
6. Task 12 (chat endpoints)
7. Task 13 (final verify)
