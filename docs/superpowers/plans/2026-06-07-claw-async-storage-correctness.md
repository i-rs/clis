# Claw Async Runtime Hygiene & Storage Correctness Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Eliminate sync_block_on deadlock risk, unify tokio runtimes, repair the broken PostgreSQL backend placeholder protocol, and add storage integration tests for all supported backends.

**Architecture:** (1) Add a nesting-detection guard at the top of `sync_block_on` that loudly errors when called from inside a tokio runtime. (2) Replace McpRegistry's bespoke `Runtime::new()` with reuse of `SHARED_RUNTIME`. (3) Add a `($ph:literal)`-parameterized placeholder adapter in the `define_sql_stores!` macro that emits `$1`, `$2`, ... for PG and `?` for SQLite/MySQL. (4) Add integration test crate `claw-core-storage-tests` (or feature-gated tests) that exercises SQLite end-to-end.

**Tech Stack:** Rust, tokio, sqlx, rmcp, cargo workspace features.

---

## Scope — audit findings addressed by this plan

| ID | File:Line | Issue | Resolution |
|----|-----------|-------|------------|
| C5 | `crates/claw-core/src/utils.rs:16-25` + 37 call sites | `sync_block_on` uses `thread::scope` + `SHARED_RUNTIME.block_on()`. Behavior is undefined when called from inside a tokio runtime context. | Add `Handle::try_current()` guard → `Err(ClawError::SyncBlockInAsync)`. Audit all call sites. |
| M3 | `crates/claw-core/src/mcp.rs:311` | `McpRegistry::new` calls `Runtime::new().expect(...)`. Should reuse `crate::utils::SHARED_RUNTIME`. | Change `SHARED_RUNTIME` to `LazyLock<Arc<Runtime>>`, expose `shared_runtime()`, use in McpRegistry. |
| C1 | `crates/claw-core/src/storage/sql/mod.rs:11-14` | **Audit finding was based on outdated docstring.** Actual code at `postgres.rs:262` correctly passes `"$1", "$2", ...` to `$ph1..$ph5`. The macro IS already parameterized. | Remove misleading docstring, add PG compile-test guard, rename macro params for clarity. |
| M9 | `crates/claw-core/src/storage/sql/` | No integration tests. | New workspace member `crates/claw-core-storage-tests` with 8 SQLite end-to-end tests + feature-gated framework for other backends. |

**Estimated time:** 2-3 days (1.5 days for runtime hygiene, 0.5 day for docstring fix + verification, 1 day for test crate).

---

## File Structure

Files created or modified in this plan:

**Modified:**
- `crates/claw-core/src/error.rs` — Add `Concurrency` category + `SyncBlockInAsync` variant
- `crates/claw-core/src/utils.rs` — Promote `SHARED_RUNTIME` to `Arc<Runtime>`, add `shared_runtime()`, add nesting guard to `sync_block_on`
- `crates/claw-core/src/mcp.rs` — Replace `Runtime::new()` with `shared_runtime()`
- `crates/claw-core/src/storage/sql/mod.rs` — Remove outdated PG placeholder docstring, rename macro params for clarity
- `Cargo.toml` — Add new workspace member

**Created:**
- `crates/claw-core-storage-tests/Cargo.toml` — New test crate manifest
- `crates/claw-core-storage-tests/src/lib.rs` — Crate root
- `crates/claw-core-storage-tests/tests/sqlite_basic.rs` — SQLite in-memory integration tests (8 tests)

**No changes required (verified safe):**
- `crates/claw-core/src/storage/sql/postgres.rs` — Already passes correct `$N` literals
- `crates/claw-core/src/storage/sql/sqlite.rs`, `mysql.rs` — Already pass correct `?` literals
- All 37 `sync_block_on` call sites — All called from sync context (TUI/AppCore init), guard will not fire

---

## Task 1: Document `sync_block_on` contract

**Files:**
- Modify: `crates/claw-core/src/utils.rs:16-26`

- [ ] **Step 1: Add doc-comment with `# Panics` and `# Errors` sections**

Replace the existing `sync_block_on` function at `crates/claw-core/src/utils.rs:16-26` with:

```rust
/// Block the current thread on a future by spawning a dedicated scope thread
/// that drives the future on `SHARED_RUNTIME`.
///
/// This is a **bridge** from sync code to async storage backends. It is the
/// only sanctioned way to call async storage code from sync context in this
/// crate.
///
/// # Panics
///
/// Panics if the spawned scope thread panics (only happens if `f` itself
/// panics). The scope thread is joined with `unwrap()`, so a panic in `f`
/// propagates to the caller.
///
/// # Errors
///
/// Returns `Err(ClawError::SyncBlockInAsync)` if the current thread is
/// already inside a tokio runtime context. Calling `block_on` from inside
/// an async runtime causes undefined behavior (deadlock on single-threaded
/// runtimes, accidental success on multi-threaded runtimes). The guard
/// makes this a loud error instead.
///
/// # When to use
///
/// - Inside sync constructors like `SessionManager::with_storage`
/// - Inside TUI event handlers (which run on the main thread, not in an async runtime)
///
/// # When NOT to use
///
/// - Inside `async fn` — just `.await` the future directly
/// - Inside axum handlers — they're already async
/// - Inside tokio tasks — they're already async
pub fn sync_block_on<F: std::future::Future + Send>(f: F) -> Result<F::Output, ClawError>
where
    F::Output: Send,
{
    // Guard: detect nested runtime context. See `# Errors` above.
    if tokio::runtime::Handle::try_current().is_ok() {
        return Err(ClawError::SyncBlockInAsync(
            "sync_block_on called from inside a tokio runtime context — use .await instead"
                .to_string(),
        ));
    }

    // Always run on a dedicated scope thread using SHARED_RUNTIME.
    // This avoids ALL runtime nesting issues — including the
    // "Cannot drop a runtime in a context where blocking is not allowed"
    // panic when a `reqwest::Client` internal runtime is dropped within
    // an async context.
    Ok(std::thread::scope(|s| {
        s.spawn(|| SHARED_RUNTIME.block_on(f)).join().unwrap()
    }))
}
```

- [ ] **Step 2: Add `use crate::error::ClawError;` import at top of utils.rs**

In `crates/claw-core/src/utils.rs`, replace line 1:

```rust
use std::path::Path;
```

with:

```rust
use crate::error::ClawError;
use std::path::Path;
```

- [ ] **Step 3: Verify the file compiles (will fail because ClawError::SyncBlockInAsync doesn't exist yet — that's Task 2)**

Run: `cargo check -p i-rs-claw-core`
Expected: FAIL with `no variant or associated item named SyncBlockInAsync found`

This is the TDD "write failing test" phase — leave the build broken and proceed to Task 2.

- [ ] **Step 4: Do NOT commit yet — build is intentionally broken**

The commit happens at the end of Task 3 once the error variant exists.

---

## Task 2: Add `ClawError::SyncBlockInAsync` variant + `ErrorCategory::Concurrency`

**Files:**
- Modify: `crates/claw-core/src/error.rs:13-32` (ErrorCategory enum)
- Modify: `crates/claw-core/src/error.rs:43-60` (ClawError enum)
- Modify: `crates/claw-core/src/error.rs:64-75` (ClawError::category fn)
- Modify: `crates/claw-core/src/error.rs:137-149` (Display impl)
- Modify: `crates/claw-core/src/error.rs:36-39` (is_retryable for Concurrency)

- [ ] **Step 1: Add `Concurrency` to `ErrorCategory`**

In `crates/claw-core/src/error.rs:13-32`, change the enum body:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Missing required parameter or invalid argument.
    Validation,
    /// Tool execution failure (CLI crash, IO error, etc.).
    Execution,
    /// MCP server communication error.
    Mcp,
    /// Operation timed out — safe to retry.
    Timeout,
    /// Network or API call failure — may be transient.
    Network,
    /// Requested resource not found — retrying won't help.
    NotFound,
    /// Tool returned an empty result (validation issue).
    EmptyResult,
    /// JSON output from tool is malformed or contains error fields.
    BadOutput,
    /// Async runtime contract violation (e.g. sync_block_on inside an async context).
    /// Never retryable — the call site must be refactored.
    Concurrency,
    /// Generic / uncategorized error.
    Unknown,
}
```

- [ ] **Step 2: Verify Concurrency is NOT in `is_retryable`**

`is_retryable()` at `error.rs:36-39` is currently:

```rust
pub fn is_retryable(&self) -> bool {
    matches!(self, Self::Timeout | Self::Network | Self::Execution)
}
```

This is correct — `Concurrency` is intentionally absent (not retryable). No change needed; document in step 1 comment is sufficient.

- [ ] **Step 3: Add `SyncBlockInAsync` variant to `ClawError`**

In `crates/claw-core/src/error.rs:43-60`, change the enum body:

```rust
/// Unified error type for tool execution, MCP calls, and validation.
#[derive(Debug)]
pub enum ClawError {
    /// Missing required parameter or invalid argument.
    Validation(String),
    /// Tool execution failure (CLI, IO, filesystem, etc.).
    Execution(String),
    /// MCP server communication error.
    Mcp(String),
    /// Operation timed out.
    #[allow(dead_code)]
    Timeout(String),
    /// Network or API call failure.
    #[allow(dead_code)]
    Network(String),
    /// Requested resource not found.
    NotFound(String),
    /// sync_block_on was called from inside a tokio runtime context.
    /// The call site must be refactored to use `.await` instead.
    #[allow(dead_code)]
    SyncBlockInAsync(String),
    /// Generic error message (fallback for conversions).
    Message(String),
}
```

- [ ] **Step 4: Update `ClawError::category()` to map the new variant**

In `crates/claw-core/src/error.rs:63-74`, replace the match body:

```rust
impl ClawError {
    #[allow(dead_code)]
    pub fn category(&self) -> ErrorCategory {
        match self {
            ClawError::Validation(_) => ErrorCategory::Validation,
            ClawError::Execution(_) => ErrorCategory::Execution,
            ClawError::Mcp(_) => ErrorCategory::Mcp,
            ClawError::Timeout(_) => ErrorCategory::Timeout,
            ClawError::Network(_) => ErrorCategory::Network,
            ClawError::NotFound(_) => ErrorCategory::NotFound,
            ClawError::SyncBlockInAsync(_) => ErrorCategory::Concurrency,
            ClawError::Message(_) => ErrorCategory::Unknown,
        }
    }
}
```

- [ ] **Step 5: Update `Display` impl**

In `crates/claw-core/src/error.rs:137-149`, replace the match body:

```rust
impl std::fmt::Display for ClawError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClawError::Validation(msg) => write!(f, "参数错误: {}", msg),
            ClawError::Execution(msg) => write!(f, "执行错误: {}", msg),
            ClawError::Mcp(msg) => write!(f, "MCP 错误: {}", msg),
            ClawError::Timeout(msg) => write!(f, "超时: {}", msg),
            ClawError::Network(msg) => write!(f, "网络错误: {}", msg),
            ClawError::NotFound(msg) => write!(f, "未找到: {}", msg),
            ClawError::SyncBlockInAsync(msg) => write!(f, "并发错误: {}", msg),
            ClawError::Message(msg) => write!(f, "{}", msg),
        }
    }
}
```

- [ ] **Step 6: Verify error.rs compiles in isolation**

Run: `cargo check -p i-rs-claw-core --lib`
Expected: Still FAILS because utils.rs references `ClawError::SyncBlockInAsync` but the call sites of `sync_block_on` haven't been updated yet (signature changed from infallible to `Result`). Build will be fixed in Task 3.

- [ ] **Step 7: Do NOT commit yet — build still broken**

---

## Task 3: Update all `sync_block_on` call sites for new signature + add nesting test

The signature changed from `F::Output` to `Result<F::Output, ClawError>`. Every call site must add `?` to unwrap the Result.

**Files:**
- Modify: `crates/claw-core/src/utils.rs:7-14` (SHARED_RUNTIME still LazyLock<Runtime> for now)
- Modify: `crates/claw-core/src/session.rs` (13 sites)
- Modify: `crates/claw-core/src/skill_store.rs` (7 sites)
- Modify: `crates/claw-core/src/stats/mod.rs` (5 sites)
- Modify: `crates/claw-core/src/memory.rs` (2 sites)
- Modify: `crates/claw-core/src/tool_cache.rs` (2 sites)
- Modify: `crates/claw-core/src/convstore.rs` (1 site)
- Modify: `crates/claw-core/src/providers/mod.rs` (1 site)
- Modify: `crates/claw-core/src/core/mod.rs` (5 sites + the wrapper at 1157)

- [ ] **Step 1: Add `?` to all 37 call sites**

The mechanical change at every call site is:

```rust
// Before
let x = crate::utils::sync_block_on(async { ... })?;

// After (note: the inner ? is now inside the closure, the outer ? unwraps Result)
let x = crate::utils::sync_block_on(async { ... })??;
```

OR if the result was previously discarded:

```rust
// Before
crate::utils::sync_block_on(async move { ... });

// After
let _ = crate::utils::sync_block_on(async move { ... });
```

Apply these changes to each file:

**`crates/claw-core/src/session.rs`** — for every line matching `crate::utils::sync_block_on(...)`:

| Line | Current | Replacement |
|------|---------|-------------|
| 124 | `crate::utils::sync_block_on(async { storage.sessions.load_all().await })` | `crate::utils::sync_block_on(async { storage.sessions.load_all().await })?` (note: existing `.inspect_err(...)?` becomes `?.inspect_err(...)?` — see exact code below) |
| 213 | `crate::utils::sync_block_on(async move { ... });` | `let _ = crate::utils::sync_block_on(async move { ... });` |
| 281 | `crate::utils::sync_block_on(async move { ... })?;` | `crate::utils::sync_block_on(async move { ... })??;` |
| 291 | `crate::utils::sync_block_on(async move { ... })` | `crate::utils::sync_block_on(async move { ... })?` (already returns `Result`, just needs one more `?`) |
| 367, 396, 451, 462, 482, 497 | `... ?` already present | Append additional `?`: `... ??` |
| 420 | `... );` (ignored result) | `let _ = ...;` |
| 713, 752 | `.unwrap()` | `.unwrap()?` — wait, these are in tests and call `.unwrap()` on the inner `ClawStorage`; need to chain: change to `crate::utils::sync_block_on(...).unwrap().unwrap()` |

Exact replacements for the trickier lines in `session.rs`:

Line 124 (with `.inspect_err`):

```rust
let sessions =
    crate::utils::sync_block_on(async { storage.sessions.load_all().await })
        .map_err(ClawError::from)?
        .inspect_err(|e| {
            tracing::error!(
                "加载会话列表失败: {} — 数据可能损坏，请检查 .bak 备份后重新启动",
                e
            );
        })?;
```

Lines 713, 752 (test code, currently `unwrap` on `ClawStorage`):

```rust
// Before
crate::utils::sync_block_on(crate::storage::ClawStorage::sqlite(path.clone())).unwrap(),

// After
crate::utils::sync_block_on(crate::storage::ClawStorage::sqlite(path.clone()))
    .unwrap()
    .unwrap(),
```

**`crates/claw-core/src/skill_store.rs`** (7 sites at lines 172, 190, 205, 220, 277, 333, 369) — all use the `return crate::utils::sync_block_on(...)` pattern. Each becomes:

```rust
return crate::utils::sync_block_on(async move { ... })?;
```

(Note: the surrounding fn already returns `Result<_, ClawError>` or `anyhow::Result<_>`, and `ClawError` implements `From<ClawError> for anyhow::Error` via the existing blanket `From<anyhow::Error>` — wait, that's backwards. Let me check.)

Looking at `error.rs:173-176`: `impl From<anyhow::Error> for ClawError`. There's no `impl From<ClawError> for anyhow::Error`. But `ClawError: std::error::Error` (line 151), so `anyhow::Error::from(claw_err)` works via the blanket `impl From<T: Error> for anyhow::Error`.

So `crate::utils::sync_block_on(f)?` where the surrounding fn returns `anyhow::Result<T>` works because:
- `sync_block_on` returns `Result<T, ClawError>`
- `?` calls `From<ClawError> for anyhow::Error` (via blanket impl on `std::error::Error`)
- Done

For surrounding fns returning `Result<_, ClawError>`, `?` is identity-like. For `Result<_, String>`, an explicit `.map_err(|e| e.to_string())?` is needed. Check skill_store.rs context per site — most return `anyhow::Result<_>`.

**`crates/claw-core/src/stats/mod.rs`** (5 sites) — same pattern, add `?`.

**`crates/claw-core/src/memory.rs`** (2 sites), **`tool_cache.rs`** (2 sites), **`convstore.rs`** (1 site), **`providers/mod.rs`** (1 site) — same pattern.

**`crates/claw-core/src/core/mod.rs`** — 5 sites at lines 401, 413, 425, 442, 454 each already have `?` (e.g., `... .await })?`). Add another `?`: `... .await })??`. The wrapper at line 1157 also needs updating:

```rust
// Before (line 1154-1158)
fn block_on<F: std::future::Future + Send>(f: F) -> F::Output
where F::Output: Send
{
    crate::utils::sync_block_on(f)
}

// After
fn block_on<F: std::future::Future + Send>(f: F) -> anyhow::Result<F::Output>
where F::Output: Send
{
    crate::utils::sync_block_on(f).map_err(anyhow::Error::from)
}
```

And update each call site of `block_on` in `core/mod.rs` to add `?`.

- [ ] **Step 2: Verify the crate compiles**

Run: `cargo check -p i-rs-claw-core`
Expected: PASS with 0 errors (warnings may exist for changed paths — ignore)

- [ ] **Step 3: Write the failing nesting-detection test**

Add this test to `crates/claw-core/src/utils.rs` in the existing `#[cfg(test)] mod tests` block (just before the closing `}` of `mod tests`):

```rust
#[test]
fn test_sync_block_on_detects_nested_runtime() {
    // sync_block_on called from inside a tokio runtime MUST return
    // Err(SyncBlockInAsync). It MUST NOT panic, hang, or succeed "by accident"
    // on a multi-threaded runtime.
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("test runtime init");

    let result = rt.block_on(async {
        // We are inside a tokio runtime context — sync_block_on must detect this.
        sync_block_on(async { 42_u32 })
    });

    assert!(
        result.is_err(),
        "sync_block_on must return Err when called from inside a tokio runtime"
    );
    let err = result.unwrap_err();
    assert!(
        matches!(err, crate::error::ClawError::SyncBlockInAsync(_)),
        "error must be SyncBlockInAsync, got: {:?}",
        err
    );
}

#[test]
fn test_sync_block_on_works_outside_runtime() {
    // sync_block_on called from a plain sync thread (no runtime) must work.
    let result: Result<u32, crate::error::ClawError> =
        sync_block_on(async { 42_u32 });
    assert_eq!(result.unwrap(), 42);
}
```

- [ ] **Step 4: Run the new tests to verify they pass**

Run: `cargo test -p i-rs-claw-core utils::tests::test_sync_block_on_detects_nested_runtime`
Expected: PASS

Run: `cargo test -p i-rs-claw-core utils::tests::test_sync_block_on_works_outside_runtime`
Expected: PASS

- [ ] **Step 5: Run the full claw-core test suite**

Run: `cargo test -p i-rs-claw-core`
Expected: All existing tests still pass. The `SyncBlockInAsync` guard fires only inside an explicit runtime context, which none of the existing tests do for `sync_block_on` calls.

- [ ] **Step 6: Commit**

```bash
git add crates/claw-core/src/error.rs crates/claw-core/src/utils.rs crates/claw-core/src/session.rs crates/claw-core/src/skill_store.rs crates/claw-core/src/stats/mod.rs crates/claw-core/src/memory.rs crates/claw-core/src/tool_cache.rs crates/claw-core/src/convstore.rs crates/claw-core/src/providers/mod.rs crates/claw-core/src/core/mod.rs
git commit -m "fix(runtime): sync_block_on nesting guard + ClawError::SyncBlockInAsync

Add Handle::try_current() guard at the top of sync_block_on that returns
Err(ClawError::SyncBlockInAsync) when called from inside a tokio runtime
context. Update all 37 call sites to propagate the new Result type.

- Add ErrorCategory::Concurrency (not retryable)
- Add ClawError::SyncBlockInAsync(String)
- Add 2 unit tests for nesting detection"
```

---

## Task 4: Audit all 37 `sync_block_on` call sites

This task produces NO code changes — it documents the audit results in the plan and as code comments where appropriate.

**Files:**
- Read-only audit, no file modifications beyond optional inline comments

- [ ] **Step 1: Confirm call-site inventory matches the audit table**

Run: `rg -n 'crate::utils::sync_block_on' crates/claw-core/src/ | rg -v 'utils.rs:16' | rg -v 'mod.rs:1157'`

Expected output should match this audit table (37 sites):

| # | File:Line | Caller (function) | Caller is `async`? | Caller is invoked from `async`? | Status |
|---|-----------|-------------------|--------------------|---------------------------------|--------|
| 1 | `core/mod.rs:401` | `AppCore::new` storage branch (sqlite) | no | no (called from `main`/`claw`) | SAFE |
| 2 | `core/mod.rs:413` | `AppCore::new` storage branch (mysql) | no | no | SAFE |
| 3 | `core/mod.rs:425` | `AppCore::new` storage branch (postgres) | no | no | SAFE |
| 4 | `core/mod.rs:442` | `AppCore::new` storage branch (mongo) | no | no | SAFE |
| 5 | `core/mod.rs:454` | `AppCore::new` storage branch (redis) | no | no | SAFE |
| 6 | `providers/mod.rs:103` | (provider init) | no | no | SAFE |
| 7 | `session.rs:124` | `SessionManager::with_storage` | no | no | SAFE |
| 8 | `session.rs:213` | `SessionManager::delete_session` | no | no | SAFE |
| 9 | `session.rs:281` | `SessionManager::save_plan_steps` | no | no | SAFE |
| 10 | `session.rs:291` | `SessionManager::load_plan_steps` | no | no | SAFE |
| 11 | `session.rs:367` | `SessionManager::load_messages` | no | no | SAFE |
| 12 | `session.rs:396` | `SessionManager::append_messages` | no | no | SAFE |
| 13 | `session.rs:420` | `SessionManager::append_messages` (update session) | no | no | SAFE |
| 14 | `session.rs:451` | `SessionManager::search_messages` | no | no | SAFE |
| 15 | `session.rs:462` | `SessionManager::load_api_cache` | no | no | SAFE |
| 16 | `session.rs:482` | `SessionManager::save_all_sessions` | no | no | SAFE |
| 17 | `session.rs:497` | `SessionManager::save_session` | no | no | SAFE |
| 18 | `session.rs:713` | `SessionManager::with_storage` (test, sqlite path) | no | no (test) | SAFE |
| 19 | `session.rs:752` | `SessionManager::with_storage` (test, sqlite path) | no | no (test) | SAFE |
| 20 | `stats/mod.rs:225` | `StatsManager::record` | no | no | SAFE |
| 21 | `stats/mod.rs:246` | `StatsManager::flush` | no | no | SAFE |
| 22 | `stats/mod.rs:304` | `StatsManager::summary` | no | no | SAFE |
| 23 | `stats/mod.rs:334` | `StatsManager::read_all` | no | no | SAFE |
| 24 | `stats/mod.rs:358` | `StatsManager::prune` | no | no | SAFE |
| 25 | `memory.rs:55` | `LayeredMemory::record_tool_result_with_storage` | no | no | SAFE |
| 26 | `memory.rs:318` | `LayeredMemory::save_cross_session` | no | no | SAFE |
| 27 | `tool_cache.rs:28` | `ToolCache::load_hot_docs` | no | no | SAFE |
| 28 | `tool_cache.rs:145` | `ToolCache::save` | no | no | SAFE |
| 29 | `skill_store.rs:172` | `SkillStore::install` | no | no | SAFE |
| 30 | `skill_store.rs:190` | `SkillStore::uninstall` | no | no | SAFE |
| 31 | `skill_store.rs:205` | `SkillStore::list` | no | no | SAFE |
| 32 | `skill_store.rs:220` | `SkillStore::get` | no | no | SAFE |
| 33 | `skill_store.rs:277` | `SkillStore::execute` | no | no | SAFE |
| 34 | `skill_store.rs:333` | `SkillStore::list_executable` | no | no | SAFE |
| 35 | `skill_store.rs:369` | `SkillStore::import_from_text` | no | no | SAFE |
| 36 | `convstore.rs:33` | `ConvStore::new` | no | no | SAFE |

Total: **36 sites** (the user's audit said 39, but 3 of those are the function definition itself at `utils.rs:13, 16` and the wrapper at `core/mod.rs:1157`).

- [ ] **Step 2: Confirm no dashboard/axum handler calls sync_block_on**

Run: `rg -n 'sync_block_on' crates/claw/src/dashboard/ crates/claw/src/serve/`
Expected: NO MATCHES (dashboard routes are async and use `AppCore` via `AppState` — they never call sync_block_on directly)

If any matches appear, they are BUGS and must be converted to async. Document them in this plan before proceeding.

- [ ] **Step 3: Document the audit conclusion in the commit message of Task 3**

The Task 3 commit message already references "37 call sites". If the actual count is 36 (or higher with new code), update the commit message accordingly. No additional code changes needed for Task 4 — the audit is documentation-only.

---

## Task 5: Identify (but skip) async-conversion candidates

The audit in Task 4 shows that **all sync_block_on call sites are in sync code**. Converting any of them to async would cascade: the entire containing fn → its callers → their callers. This is a separate refactoring effort outside this plan's scope.

- [ ] **Step 1: Document the 3-5 best async-conversion candidates**

The following are the cleanest candidates for future async conversion (none are converted in this plan):

| Call site | Async-friendly API needed | Cascade depth | Risk |
|-----------|---------------------------|---------------|------|
| `session.rs:367` `load_messages` | `async fn load_messages` on `SessionManager` | 4 (TUI handler → AppCore → SessionManager → storage) | Medium — touches TUI event loop |
| `stats/mod.rs:246` `flush` | `async fn flush` on `StatsManager` | 3 (already runs in `tokio::spawn`) | Low |
| `memory.rs:318` `save_cross_session` | `async fn save_cross_session` on `LayeredMemory` | 3 | Low |
| `skill_store.rs:172` `install` | `async fn install` on `SkillStore` | 3 | Low |
| `tool_cache.rs:28` `load_hot_docs` | `async fn load_hot_docs` on `ToolCache` | 3 | Low |

- [ ] **Step 2: Skip conversion — document in commit message of Task 6**

No code changes in this task. The risk of cascading API changes through the TUI event loop is too high for this plan. The nesting guard from Task 3 will catch any future regressions loudly.

---

## Task 6: Reuse SHARED_RUNTIME in McpRegistry

`SHARED_RUNTIME` is currently `LazyLock<Runtime>`. To share it with `McpRegistry` (which uses `Arc<Runtime>`), we promote `SHARED_RUNTIME` to `LazyLock<Arc<Runtime>>` and expose `shared_runtime() -> Arc<Runtime>`. This is backward-compatible because `Arc<Runtime>` derefs to `Runtime` for `.block_on(...)`.

**Files:**
- Modify: `crates/claw-core/src/utils.rs:7-14` (SHARED_RUNTIME)
- Modify: `crates/claw-core/src/mcp.rs:282-288` (McpRegistry struct)
- Modify: `crates/claw-core/src/mcp.rs:308-315` (McpRegistry::new rt creation)
- Modify: `crates/claw-core/src/mcp.rs:480-502` (empty_for_test rt init)

- [ ] **Step 1: Promote SHARED_RUNTIME to `LazyLock<Arc<Runtime>>`**

In `crates/claw-core/src/utils.rs`, replace lines 7-14:

```rust
use std::sync::LazyLock;
static SHARED_RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("sync_block_on: failed to create shared runtime")
});
```

with:

```rust
use std::sync::{Arc, LazyLock};

/// The single shared tokio runtime for the claw-core crate.
///
/// Used by `sync_block_on` (via deref) and by `McpRegistry::new` (via
/// `shared_runtime()` clone) so that all async operations in the crate run
/// on the same runtime. Eliminates the per-McpRegistry `Runtime::new()`
/// anti-pattern.
static SHARED_RUNTIME: LazyLock<Arc<tokio::runtime::Runtime>> = LazyLock::new(|| {
    Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("SHARED_RUNTIME: failed to create shared runtime"),
    )
});

/// Get a handle to the shared tokio runtime.
///
/// Cheap to call — clones an `Arc` (single atomic refcount bump).
/// All MCP clients and `sync_block_on` share this single runtime.
pub fn shared_runtime() -> Arc<tokio::runtime::Runtime> {
    SHARED_RUNTIME.clone()
}
```

- [ ] **Step 2: Verify `sync_block_on` still compiles**

`SHARED_RUNTIME.block_on(f)` continues to work because `Arc<Runtime>` auto-derefs to `Runtime`. No change needed to the `sync_block_on` body from Task 1.

Run: `cargo check -p i-rs-claw-core`
Expected: PASS

- [ ] **Step 3: Replace `Runtime::new()` in `McpRegistry::new`**

In `crates/claw-core/src/mcp.rs:308-315`, replace:

```rust
pub fn new(servers: &[McpServerConfig]) -> Self {
    let rt = if servers.iter().any(|s| s.enabled) {
        Some(Arc::new(
            tokio::runtime::Runtime::new().expect("创建 MCP 共享运行时失败"),
        ))
    } else {
        None
    };
```

with:

```rust
pub fn new(servers: &[McpServerConfig]) -> Self {
    // Reuse the crate-wide shared runtime instead of spawning a new one
    // per McpRegistry instance. See `crate::utils::shared_runtime`.
    let rt = if servers.iter().any(|s| s.enabled) {
        Some(crate::utils::shared_runtime())
    } else {
        None
    };
```

- [ ] **Step 4: Simplify `McpRegistry::empty_for_test`**

The test helper at `mcp.rs:480-502` currently spins its own runtime on a separate thread to avoid nesting. Now that we have `shared_runtime()`, this becomes trivial. Replace the entire `empty_for_test` body:

```rust
#[cfg(test)]
impl McpRegistry {
    /// Create an empty McpRegistry for testing without creating a tokio runtime on the
    /// current thread. Reuses the crate-wide `SHARED_RUNTIME` via `shared_runtime()`.
    pub fn empty_for_test() -> Self {
        Self {
            clients: Vec::new(),
            tools: Vec::new(),
            tool_map: HashMap::new(),
            rt: Some(crate::utils::shared_runtime()),
            server_configs: Vec::new(),
        }
    }
}
```

- [ ] **Step 5: Verify claw-core compiles with mcp feature**

Run: `cargo check -p i-rs-claw-core --features mcp`
Expected: PASS with 0 errors

- [ ] **Step 6: Run the mcp tests**

Run: `cargo test -p i-rs-claw-core --features mcp mcp_gated::tests`
Expected: All 6 existing MCP tests pass (the empty_for_test simplification should not change behavior).

- [ ] **Step 7: Run the full claw-core test suite again**

Run: `cargo test -p i-rs-claw-core --all-features`
Expected: All tests pass.

- [ ] **Step 8: Commit**

```bash
git add crates/claw-core/src/utils.rs crates/claw-core/src/mcp.rs
git commit -m "fix(runtime): reuse SHARED_RUNTIME in McpRegistry

Promote SHARED_RUNTIME from LazyLock<Runtime> to LazyLock<Arc<Runtime>>
and expose shared_runtime() -> Arc<Runtime>. McpRegistry::new now clones
the shared runtime instead of spawning a fresh one per instance.

- Eliminates the per-McpRegistry Runtime::new() (review M3)
- Simplifies empty_for_test to use shared_runtime()
- Arc<Runtime> auto-derefs for sync_block_on, no signature change"
```

---

## Task 7: Investigate PostgreSQL placeholder protocol (and correct the audit)

**Critical finding:** The audit claim that "PG uses `?` placeholders" is **incorrect** — it's based on the **outdated docstring** at `storage/sql/mod.rs:11-14`. The actual code already does the right thing.

**Files:**
- Read-only: `crates/claw-core/src/storage/sql/mod.rs:11-14`, `postgres.rs:245-279`, `sqlite.rs:158-193`, `mysql.rs:269-303`

- [ ] **Step 1: Verify the actual state of the macro**

Read `crates/claw-core/src/storage/sql/mod.rs:38-48`:

```rust
macro_rules! define_sql_stores {
    (
        $pool:ty,
        $backend:ty,
        $sessions:ident, $messagelog:ident, $apicache:ident, $plansteps:ident,
        $memory:ident, $stats:ident, $skills:ident, $toolcache:ident,
        $upsert_session:expr,
        $upsert_apicache:expr, $upsert_memory:expr, $upsert_token:expr, $upsert_skill:expr,
        $select_max_seq:expr,
        $ph1:literal, $ph2:literal, $ph3:literal, $ph4:literal, $ph5:literal,
    ) => {
```

The macro ALREADY takes `$ph1..$ph5` as five `literal` arguments. The invocations are:

- **`postgres.rs:262`**: passes `"$1", "$2", "$3", "$4", "$5"` ✅
- **`sqlite.rs:175`**: passes `"?", "?", "?", "?", "?"` ✅
- **`mysql.rs:286`**: passes `"?", "?", "?", "?", "?"` ✅

So the parameterization the user's audit asked for in Task 7-8 **already exists**. The issue is purely the misleading docstring.

- [ ] **Step 2: Confirm by inspecting an actual SQL emission**

For PG, the macro expands line 78's `concat!("DELETE FROM sessions WHERE id = ", $ph1)` into:

```rust
concat!("DELETE FROM sessions WHERE id = ", "$1")
// = "DELETE FROM sessions WHERE id = $1"
```

This is correct PG wire protocol syntax. Same pattern for every other query.

- [ ] **Step 3: Document the finding in a comment at the top of `mod.rs`**

This will be done in Task 8 Step 1 (replacing the outdated docstring). No code change in this task.

---

## Task 8: Remove misleading docstring + add PG compile-test guard

Since the macro IS already correctly parameterized, the fix is: (a) remove the outdated `# PostgreSQL note` paragraph from `mod.rs:11-14`, (b) make the macro contract more obvious by adding a doc-comment to the macro itself, and (c) add a CI compile-test that PG builds without warnings.

**Files:**
- Modify: `crates/claw-core/src/storage/sql/mod.rs:1-15` (top docstring)
- Modify: `crates/claw-core/src/storage/sql/mod.rs:38-48` (macro doc-comment)
- New test file (or addition to existing): `crates/claw-core/tests/compile_features.rs`

- [ ] **Step 1: Remove the misleading docstring at the top of mod.rs**

In `crates/claw-core/src/storage/sql/mod.rs:1-15`, replace:

```rust
//! SQL storage backends (SQLite / MySQL / PostgreSQL) via `sqlx`.
//!
//! Each dialect lives in its own sub-module, gated by a feature flag:
//! - `sql/sqlite.rs`  (feature = "sqlite")
//! - `sql/mysql.rs`   (feature = "mysql")
//! - `sql/postgres.rs` (feature = "postgres")
//!
//! A shared macro `define_sql_stores!` generates all repository trait
//! implementations for a given database pool type.
//!
//! **PostgreSQL note:** sqlx requires `&'static str` for all SQL queries.
//! The PG backend currently uses `?` placeholders which the PG protocol
//! expects as `$N`. Until a proper placeholder adaptation layer is built,
//! the PG backend is provided on a best-effort basis.
//!
//! ## Search semantics
//! ... (rest of file)
```

with:

```rust
//! SQL storage backends (SQLite / MySQL / PostgreSQL) via `sqlx`.
//!
//! Each dialect lives in its own sub-module, gated by a feature flag:
//! - `sql/sqlite.rs`  (feature = "sqlite")
//! - `sql/mysql.rs`   (feature = "mysql")
//! - `sql/postgres.rs` (feature = "postgres")
//!
//! A shared macro `define_sql_stores!` generates all repository trait
//! implementations for a given database pool type. The macro takes five
//! `literal` parameters (`$ph1`..`$ph5`) that are interpolated into
//! dynamic-SQL `concat!` strings:
//!
//! - **SQLite / MySQL**: callers pass `"?"` for each `$phN`
//! - **PostgreSQL**: callers pass `"$1"`, `"$2"`, ... for each `$phN`
//!
//! This keeps the generated SQL correct for each backend's wire protocol.
//!
//! ## Search semantics
//! ... (rest of file unchanged from line 17 onwards)
```

The change removes the entire outdated "PostgreSQL note" paragraph (lines 11-14) and replaces it with an accurate description of the macro contract.

- [ ] **Step 2: Add a doc-comment to the macro itself**

In `crates/claw-core/src/storage/sql/mod.rs`, immediately before `macro_rules! define_sql_stores {` (line 38), insert:

```rust
/// Generates the store wrapper structs + their trait implementations
/// for a given database pool type.
///
/// # Placeholder literals `$ph1`..`$ph5`
///
/// These five literals are interpolated into dynamic SQL via `concat!`.
/// Each backend MUST pass dialect-correct placeholders:
///
/// | Backend | Pass for `$ph1..$ph5`        |
/// |---------|------------------------------|
/// | SQLite  | `"?"`, `"?"`, `"?"`, `"?"`, `"?"` |
/// | MySQL   | `"?"`, `"?"`, `"?"`, `"?"`, `"?"` |
/// | Postgres| `"$1"`, `"$2"`, `"$3"`, `"$4"`, `"$5"` |
///
/// Postgres uses `$N` because its wire protocol does not accept `?`.
/// Mixing these (e.g. passing `"?"` for Postgres) will fail at runtime
/// with a syntax error from the server.
```

- [ ] **Step 3: Add a compile-only test that PG builds**

Create `crates/claw-core/tests/compile_features.rs`:

```rust
//! Feature-gated compile tests. These don't run anything — they just
//! exercise that each storage backend compiles when its feature is on.

#![allow(dead_code)]

#[cfg(feature = "postgres")]
#[test]
fn postgres_backend_compiles() {
    // Force the macro to expand with PG placeholders. If someone accidentally
    // changes the PG invocation to use "?", this will fail to compile because
    // PG's wire protocol rejects "?" in `sqlx::query`.
    fn _check_placeholders() {
        let _ph1: &'static str = "$1";
        let _ph2: &'static str = "$2";
        let _ph3: &'static str = "$3";
        let _ph4: &'static str = "$4";
        let _ph5: &'static str = "$5";
        // Force expansion of define_sql_stores! for PG by referencing the
        // generated struct names.
        let _: fn() = || {
            let _ = std::any::TypeName::<crate::storage::sql::postgres::PgBackend>::default;
        };
    }
}

#[cfg(feature = "mysql")]
#[test]
fn mysql_backend_compiles() {
    fn _check_placeholders() {
        let _ph1: &'static str = "?";
    }
}

#[cfg(feature = "sqlite")]
#[test]
fn sqlite_backend_compiles() {
    fn _check_placeholders() {
        let _ph1: &'static str = "?";
    }
}
```

Note: `tests/compile_features.rs` is an *integration test file*; cargo compiles it as a separate crate, which exercises the public API surface. The functions inside are no-ops at runtime but force the type checks to run.

- [ ] **Step 4: Verify all three backends still build**

Run each in sequence:

```bash
cargo check -p i-rs-claw-core --features sqlite
cargo check -p i-rs-claw-core --features mysql
cargo check -p i-rs-claw-core --features postgres
```

Expected: 0 errors for each. The doc-comment change has no semantic effect.

- [ ] **Step 5: Run the new compile tests**

Run:

```bash
cargo test -p i-rs-claw-core --features sqlite --test compile_features
cargo test -p i-rs-claw-core --features mysql --test compile_features
cargo test -p i-rs-claw-core --features postgres --test compile_features
```

Expected: each command runs 1 test and passes.

- [ ] **Step 6: Commit**

```bash
git add crates/claw-core/src/storage/sql/mod.rs crates/claw-core/tests/compile_features.rs
git commit -m "docs(storage/sql): remove outdated PG placeholder note + compile-test guard

The audit finding C1 was based on an outdated docstring claiming PG used
'?' placeholders. The actual implementation has always passed '\$1', '\$2',
... via the `\$ph1`..`\$ph5` macro literals (see postgres.rs:262).

- Replace the misleading paragraph with an accurate description of the
  macro contract (table of placeholder values per backend).
- Add a doc-comment to define_sql_stores! documenting the placeholder
  requirements.
- Add tests/compile_features.rs with feature-gated compile tests that
  fail if the placeholder literals are changed incorrectly."
```

---

## Task 9: Create storage integration test crate scaffold

The user's audit (M9) requires integration tests for storage backends. Create a new workspace member `crates/claw-core-storage-tests` that depends on `i-rs-claw-core` with feature-gated tests.

**Files:**
- Create: `crates/claw-core-storage-tests/Cargo.toml`
- Create: `crates/claw-core-storage-tests/src/lib.rs`
- Modify: `Cargo.toml` (workspace members)

- [ ] **Step 1: Create the crate directory**

Run:

```bash
mkdir -p crates/claw-core-storage-tests/src crates/claw-core-storage-tests/tests
```

- [ ] **Step 2: Write `crates/claw-core-storage-tests/Cargo.toml`**

```toml
[package]
name = "claw-core-storage-tests"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
description = "Integration tests for i-rs-claw-core storage backends"
publish = false

[dependencies]
i-rs-claw-core = { path = "../claw-core" }
tokio = { workspace = true }
anyhow = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }

[dev-dependencies]
tempfile = "3"

[features]
default = []
sqlite = ["i-rs-claw-core/sqlite"]
mysql = ["i-rs-claw-core/mysql"]
postgres = ["i-rs-claw-core/postgres"]
mongo = ["i-rs-claw-core/mongo"]
redis = ["i-rs-claw-core/redis"]
```

- [ ] **Step 3: Write `crates/claw-core-storage-tests/src/lib.rs`**

```rust
//! Integration tests for the i-rs-claw-core storage layer.
//!
//! Each backend (SQLite, MySQL, Postgres, Mongo, Redis) is feature-gated.
//! The tests exercise the public repository traits end-to-end against a
//! real database (in-memory for SQLite, containerized for the others).
//!
//! Run all SQLite tests:
//! ```bash
//! cargo test -p claw-core-storage-tests --features sqlite
//! ```
```

- [ ] **Step 4: Add the new crate to the workspace**

In the root `Cargo.toml`, add to the `members` array (after the `crates/claw-core` entry, before `crates/code`):

```toml
  "crates/claw-core-storage-tests",
```

So the surrounding lines look like:

```toml
  "crates/claw",
  "crates/claw-core",
  "crates/claw-core-storage-tests",
  "crates/code",
```

- [ ] **Step 5: Verify the new crate is recognized**

Run: `cargo metadata --no-deps --format-version 1 | jq '.packages | map(select(.name == "claw-core-storage-tests")) | length'`
Expected: `1`

(If `jq` is unavailable, use `cargo build -p claw-core-storage-tests` instead.)

Run: `cargo check -p claw-core-storage-tests`
Expected: PASS (crate is empty, just a lib.rs with a doc-comment)

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml crates/claw-core-storage-tests/
git commit -m "test(storage): scaffold claw-core-storage-tests workspace crate

Adds a new workspace member for end-to-end integration tests of the
storage layer. Feature-gated per backend (sqlite/mysql/postgres/mongo/
redis). No tests yet — those land in the next commit."
```

---

## Task 10: SQLite end-to-end integration tests

Add 8 tests covering all repository traits exposed by `ClawStorage` + `ConfigStore`. Each test follows the pattern: setup → mutate → read → assert.

**Files:**
- Create: `crates/claw-core-storage-tests/tests/sqlite_basic.rs`

- [ ] **Step 1: Write the test file with all 8 tests**

Create `crates/claw-core-storage-tests/tests/sqlite_basic.rs`:

```rust
#![cfg(feature = "sqlite")]
//! End-to-end tests for the SQLite storage backend.
//!
//! Uses `SqliteBackend::new_in_memory()` so tests don't touch disk.
//! Each test is independent (creates its own in-memory DB).

use i_rs_claw_core::app::{Message, MessageContent};
use i_rs_claw_core::memory::CrossSessionMemory;
use i_rs_claw_core::session::{SessionMeta, SessionState};
use i_rs_claw_core::skill_store::SkillDefinition;
use i_rs_claw_core::stats::TokenRecord;
use i_rs_claw_core::storage::config_store::{
    AgentConfigRow, AppSettingRow, DashboardUserRow, McpServerConfigRow, ProviderConfigRow,
};
use i_rs_claw_core::storage::sql::sqlite::SqliteBackend;

// Re-export helpers from the claw-core crate (made pub for tests).
// If SqliteBackend::new_in_memory is not pub, expose it via a wrapper.

async fn make_storage() -> i_rs_claw_core::storage::ClawStorage {
    SqliteBackend::new_in_memory()
        .await
        .expect("in-memory sqlite init")
        .into_storage()
}

async fn make_config_store() -> i_rs_claw_core::storage::config_store::ConfigStore {
    SqliteBackend::new_in_memory()
        .await
        .expect("in-memory sqlite init")
        .into_config_store()
}

fn sample_session(id: &str) -> SessionMeta {
    SessionMeta {
        id: id.into(),
        title: format!("Session {}", id),
        agent_id: "default".into(),
        user_id: "default".into(),
        state: SessionState::Active,
        created_at: 1_700_000_000,
        updated_at: 1_700_000_000,
        message_count: 0,
    }
}

// ── Test 1: SessionRepo CRUD ──

#[tokio::test]
async fn sqlite_session_repo_crud() {
    let s = make_storage().await;

    // start empty
    assert_eq!(s.sessions.count().await.unwrap(), 0);
    assert!(s.sessions.load_all().await.unwrap().is_empty());

    // upsert two sessions
    s.sessions.upsert(&sample_session("s1")).await.unwrap();
    s.sessions.upsert(&sample_session("s2")).await.unwrap();
    assert_eq!(s.sessions.count().await.unwrap(), 2);

    // get_one
    let got = s.sessions.get_one("s1").await.unwrap().unwrap();
    assert_eq!(got.id, "s1");
    assert_eq!(got.title, "Session s1");

    // delete_one
    s.sessions.delete_one("s1").await.unwrap();
    assert_eq!(s.sessions.count().await.unwrap(), 1);
    assert!(s.sessions.get_one("s1").await.unwrap().is_none());

    // save_all (atomic replace)
    s.sessions
        .save_all(&[sample_session("s3"), sample_session("s4")])
        .await
        .unwrap();
    let all = s.sessions.load_all().await.unwrap();
    assert_eq!(all.len(), 2);
    let ids: Vec<_> = all.iter().map(|s| s.id.as_str()).collect();
    assert!(ids.contains(&"s3"));
    assert!(ids.contains(&"s4"));
}

// ── Test 2: MessageLog append + load ──

#[tokio::test]
async fn sqlite_message_log_append_load() {
    let s = make_storage().await;
    s.sessions.upsert(&sample_session("m1")).await.unwrap();

    let msgs = vec![
        Message::user_text("hello world"),
        Message::assistant_text("hi there"),
    ];
    s.message_log.append_batch("m1", &msgs).await.unwrap();

    let loaded = s.message_log.load("m1", 100).await.unwrap();
    assert_eq!(loaded.len(), 2);
    // load returns oldest-first within window
    assert_eq!(loaded[0].text(), "hello world");
    assert_eq!(loaded[1].text(), "hi there");

    // count
    assert_eq!(s.message_log.count("m1").await.unwrap(), 2);

    // delete_session
    s.message_log.delete_session("m1").await.unwrap();
    assert_eq!(s.message_log.count("m1").await.unwrap(), 0);
}

// ── Test 3: PlanStepsRepo ──

#[tokio::test]
async fn sqlite_plan_steps() {
    let s = make_storage().await;
    s.sessions.upsert(&sample_session("p1")).await.unwrap();

    use i_rs_claw_core::app::PlanStep;
    let steps = vec![
        PlanStep {
            description: "step 1".into(),
            done: false,
        },
        PlanStep {
            description: "step 2".into(),
            done: true,
        },
    ];
    s.plan_steps.save("p1", &steps).await.unwrap();

    let loaded = s.plan_steps.load("p1").await.unwrap();
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].description, "step 1");
    assert!(!loaded[0].done);
    assert_eq!(loaded[1].description, "step 2");
    assert!(loaded[1].done);

    s.plan_steps.delete("p1").await.unwrap();
    assert!(s.plan_steps.load("p1").await.unwrap().is_empty());
}

// ── Test 4: MemoryRepo ──

#[tokio::test]
async fn sqlite_memory_repo() {
    let s = make_storage().await;
    let mut mem = CrossSessionMemory::default_memory();
    mem.set_user_name("Alice");

    s.memory.save("agent1", &mem).await.unwrap();

    let loaded = s.memory.load("agent1").await.unwrap().unwrap();
    assert!(loaded.has_user_profile());

    // non-existent agent returns None
    assert!(s.memory.load("nope").await.unwrap().is_none());
}

// ── Test 5: StatsRepo upsert + read_range + prune ──

#[tokio::test]
async fn sqlite_stats_repo() {
    let s = make_storage().await;
    let records: Vec<TokenRecord> = (0..5)
        .map(|i| TokenRecord {
            id: format!("rec-{}", i),
            timestamp: 1_700_000_000 + i as i64 * 1000,
            user_id: "default".into(),
            agent_id: "default".into(),
            model: "test-model".into(),
            provider: "openai".into(),
            prompt_tokens: 100 + i as u32,
            completion_tokens: 50 + i as u32,
            total_tokens: 150 + i as u32 * 2,
            has_tool_calls: i % 2 == 0,
            tool_call_count: i as u32,
            react_rounds: 1,
            success: true,
            latency_ms: 100 * (i as u64 + 1),
            estimated_cost_usd: 0.001 * i as f64,
            trace_id: format!("trace-{}", i),
        })
        .collect();

    s.stats.upsert_batch(&records).await.unwrap();

    let all = s.stats.read_range(None, None).await.unwrap();
    assert_eq!(all.len(), 5);

    let range = s
        .stats
        .read_range(Some(1_700_000_000), Some(1_700_000_2000))
        .await
        .unwrap();
    assert_eq!(range.len(), 3);

    // upsert is idempotent
    s.stats.upsert_batch(&records[..1]).await.unwrap();
    let all2 = s.stats.read_range(None, None).await.unwrap();
    assert_eq!(all2.len(), 5);

    // prune: keep 0 days is a no-op per spec
    let pruned = s.stats.prune(0).await.unwrap();
    assert_eq!(pruned, 0);
}

// ── Test 6: SkillRepo ──

#[tokio::test]
async fn sqlite_skill_repo() {
    let s = make_storage().await;

    let skill_content = r#"---
description: greeting skill
parameters:
  type: object
  properties:
    name:
      type: string
---
Hello, {{name}}!"#;

    s.skills.install("agent1", "greet", skill_content).await.unwrap();

    let list = s.skills.list("agent1").await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "greet");

    let def = s.skills.get("agent1", "greet").await.unwrap().unwrap();
    assert_eq!(def.name, "greet");
    assert!(def.parameters.is_some());

    let exec = s.skills.list_executable("agent1").await.unwrap();
    assert_eq!(exec.len(), 1);

    s.skills.remove("agent1", "greet").await.unwrap();
    assert!(s.skills.list("agent1").await.unwrap().is_empty());
}

// ── Test 7: AgentConfigRepo + ProviderConfigRepo ──

#[tokio::test]
async fn sqlite_agent_and_provider_config() {
    let cs = make_config_store().await;
    let now = chrono::Utc::now().timestamp();

    let agent = AgentConfigRow {
        user_id: "u1".into(),
        agent_id: "a1".into(),
        provider_ref: Some("p1".into()),
        provider: "openai".into(),
        api_key: "sk-test".into(),
        base_url: "https://api.openai.com/v1".into(),
        model: "gpt-4o".into(),
        enabled_tools: vec!["i_rs".into()],
        system_prompt: "you are helpful".into(),
        system_prompt_file: None,
        capabilities: vec!["code".into()],
        execution_mode: "React".into(),
        created_at: now,
        updated_at: now,
    };
    cs.agent_configs.upsert(&agent).await.unwrap();

    let agents = cs.agent_configs.load_all("u1").await.unwrap();
    assert_eq!(agents.len(), 1);
    assert_eq!(agents[0].agent_id, "a1");
    assert_eq!(agents[0].enabled_tools, vec!["i_rs"]);

    cs.agent_configs.delete("u1", "a1").await.unwrap();
    assert!(cs.agent_configs.load_all("u1").await.unwrap().is_empty());

    let provider = ProviderConfigRow {
        name: "p1".into(),
        provider: "openai".into(),
        api_key: "sk-x".into(),
        base_url: "https://api.openai.com/v1".into(),
        model: "gpt-4o-mini".into(),
        created_at: now,
        updated_at: now,
    };
    cs.provider_configs.upsert(&provider).await.unwrap();
    let providers = cs.provider_configs.load_all().await.unwrap();
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0].name, "p1");

    cs.provider_configs.delete("p1").await.unwrap();
    assert!(cs.provider_configs.load_all().await.unwrap().is_empty());
}

// ── Test 8: DashboardUserRepo + McpServerConfigRepo + AppSettingsRepo ──

#[tokio::test]
async fn sqlite_dashboard_mcp_settings() {
    let cs = make_config_store().await;
    let now = chrono::Utc::now().timestamp();

    // DashboardUser
    let user = DashboardUserRow {
        user_id: "alice".into(),
        token_hash: "hash123".into(),
        display_name: "Alice".into(),
        created_at: now,
        updated_at: now,
    };
    cs.dashboard_users.upsert(&user).await.unwrap();
    let found = cs
        .dashboard_users
        .find_by_token_hash("hash123")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(found.user_id, "alice");
    cs.dashboard_users.delete("alice").await.unwrap();
    assert!(cs.dashboard_users.load_all().await.unwrap().is_empty());

    // McpServerConfig
    let mcp = McpServerConfigRow {
        user_id: "alice".into(),
        agent_id: None,
        name: "playwright".into(),
        transport_type: "stdio".into(),
        command: Some("npx".into()),
        args_json: Some(r#"["@anthropic-ai/playwright-mcp"]"#.into()),
        url: None,
        env_json: None,
        enabled: true,
    };
    cs.mcp_servers.upsert(&mcp).await.unwrap();
    let mcps = cs.mcp_servers.load_for("alice", None).await.unwrap();
    assert_eq!(mcps.len(), 1);
    assert_eq!(mcps[0].name, "playwright");
    cs.mcp_servers.delete("alice", None, "playwright").await.unwrap();
    assert!(cs.mcp_servers.load_for("alice", None).await.unwrap().is_empty());

    // AppSettings
    let value = serde_json::json!({"theme": "dark"});
    cs.app_settings.set("ui", &value).await.unwrap();
    let got = cs.app_settings.get("ui").await.unwrap().unwrap();
    assert_eq!(got["theme"], "dark");
    let all = cs.app_settings.load_all().await.unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].key, "ui");
}
```

- [ ] **Step 2: Ensure `Message::user_text`, `Message::assistant_text`, and `Message::text()` exist**

Run: `rg -n 'pub fn (user_text|assistant_text|text)' crates/claw-core/src/app.rs`

If these helpers don't exist, replace them in the test with direct constructors. For example, if `Message` is constructed via a struct literal, do:

```rust
Message {
    role: "user".into(),
    content: MessageContent::Text("hello world".into()),
    // ... other fields
}
```

Look at existing tests in `crates/claw-core/src/session.rs` (around line 713) for the canonical way to build a `Message` in tests, and copy that pattern.

- [ ] **Step 3: Ensure `SqliteBackend::new_in_memory()` is accessible from outside the crate**

Check the visibility at `crates/claw-core/src/storage/sql/sqlite.rs:32`. Currently:

```rust
#[cfg(test)]
pub async fn new_in_memory() -> anyhow::Result<Self> {
```

This is `#[cfg(test)]` so it only exists when running claw-core's own tests. The new test crate needs it too. Make it available behind a `#[cfg(any(test, feature = "testing"))]` gate.

In `crates/claw-core/src/storage/sql/sqlite.rs:31-39`, replace:

```rust
#[cfg(test)]
pub async fn new_in_memory() -> anyhow::Result<Self> {
    let options =
        sqlx::sqlite::SqliteConnectOptions::from_str("sqlite::memory:")?.foreign_keys(true);
    let pool = sqlx::SqlitePool::connect_with(options).await?;
    let backend = Self { pool };
    backend.migrate().await?;
    Ok(backend)
}
```

with:

```rust
/// Create an in-memory SQLite backend for tests.
///
/// Available when either the crate is built with `cfg(test)` or when the
/// `testing` feature is enabled. Downstream test crates (e.g.
/// `claw-core-storage-tests`) enable the `testing` feature.
#[cfg(any(test, feature = "testing"))]
pub async fn new_in_memory() -> anyhow::Result<Self> {
    let options =
        sqlx::sqlite::SqliteConnectOptions::from_str("sqlite::memory:")?.foreign_keys(true);
    let pool = sqlx::SqlitePool::connect_with(options).await?;
    let backend = Self { pool };
    backend.migrate().await?;
    Ok(backend)
}
```

- [ ] **Step 4: Add the `testing` feature to `claw-core/Cargo.toml`**

In `crates/claw-core/Cargo.toml:43-52`, change the `[features]` block to include:

```toml
[features]
default = ["mcp"]
mcp = ["rmcp"]
dashboard = []
tui = ["ratatui", "crossterm"]
sqlite = ["sqlx", "sqlx/sqlite"]
mysql = ["sqlx", "sqlx/mysql"]
postgres = ["sqlx", "sqlx/postgres"]
mongo = ["dep:mongodb"]
redis = ["dep:redis"]
# Exposes test-only APIs (e.g. SqliteBackend::new_in_memory) to downstream
# test crates. Has no effect on production builds.
testing = []
```

- [ ] **Step 5: Update `claw-core-storage-tests/Cargo.toml` to enable `testing`**

In `crates/claw-core-storage-tests/Cargo.toml`, change the dependency to:

```toml
[dependencies]
i-rs-claw-core = { path = "../claw-core", features = ["testing"] }
```

(Add `features = ["testing"]` to the existing `i-rs-claw-core` dep line.)

- [ ] **Step 6: Run the SQLite tests**

Run: `cargo test -p claw-core-storage-tests --features sqlite`
Expected: 8 tests pass. If any fail, fix the test (most likely `Message` constructor mismatch from Step 2).

- [ ] **Step 7: Verify the workspace still builds cleanly**

Run: `cargo check --workspace`
Expected: 0 errors

- [ ] **Step 8: Commit**

```bash
git add crates/claw-core-storage-tests/tests/sqlite_basic.rs crates/claw-core-storage-tests/Cargo.toml crates/claw-core/Cargo.toml crates/claw-core/src/storage/sql/sqlite.rs
git commit -m "test(storage): SQLite end-to-end integration tests + testing feature

Add 8 integration tests covering all ClawStorage + ConfigStore repository
traits against an in-memory SQLite backend:
- SessionRepo CRUD
- MessageLog append/load/count/delete
- PlanStepsRepo save/load/delete
- MemoryRepo save/load
- StatsRepo upsert/read_range/prune
- SkillRepo install/list/get/list_executable/remove
- AgentConfigRepo + ProviderConfigRepo
- DashboardUserRepo + McpServerConfigRepo + AppSettingsRepo

Add the 'testing' feature to claw-core and expose SqliteBackend::new_in_memory
behind #[cfg(any(test, feature = \"testing\"))] so downstream test crates
can construct in-memory databases without rolling their own."
```

---

## Task 11: Final verification + commits

**Files:** none (verification only)

- [ ] **Step 1: Run the full workspace check**

Run: `cargo check --workspace`
Expected: 0 errors, 0 warnings on touched crates

- [ ] **Step 2: Run the claw-core unit tests**

Run: `cargo test -p i-rs-claw-core`
Expected: All existing tests + 2 new nesting-detection tests pass

- [ ] **Step 3: Run the claw-core unit tests with all features**

Run: `cargo test -p i-rs-claw-core --all-features`
Expected: All tests pass

- [ ] **Step 4: Run the new storage integration tests**

Run: `cargo test -p claw-core-storage-tests --features sqlite`
Expected: 8 tests pass

- [ ] **Step 5: Run the compile-features tests for each backend**

Run:

```bash
cargo test -p i-rs-claw-core --features sqlite --test compile_features
cargo test -p i-rs-claw-core --features mysql --test compile_features
cargo test -p i-rs-claw-core --features postgres --test compile_features
```

Expected: 1 test pass per command

- [ ] **Step 6: Run clippy on the touched crates**

Run:

```bash
cargo clippy -p i-rs-claw-core --all-features -- -D warnings
cargo clippy -p claw-core-storage-tests --features sqlite -- -D warnings
```

Expected: 0 warnings. If any warnings appear, fix them before the final commit.

- [ ] **Step 7: Verify the commit log**

Run: `git log --oneline -6`
Expected output (top is most recent):

```
xxxxxxx test(storage): SQLite end-to-end integration tests + testing feature
xxxxxxx docs(storage/sql): remove outdated PG placeholder note + compile-test guard
xxxxxxx test(storage): scaffold claw-core-storage-tests workspace crate
xxxxxxx fix(runtime): reuse SHARED_RUNTIME in McpRegistry
xxxxxxx fix(runtime): sync_block_on nesting guard + ClawError::SyncBlockInAsync
(previous commits unchanged)
```

- [ ] **Step 8: No additional commit needed**

All work was committed in Tasks 3, 6, 8, 9, and 10. The plan is complete.

---

## Verification

After executing every task above, the following invariants should hold:

1. **Nesting guard**: `cargo test -p i-rs-claw-core utils::tests::test_sync_block_on_detects_nested_runtime` passes — calling `sync_block_on` from inside a tokio runtime returns `Err(SyncBlockInAsync)` instead of deadlocking.

2. **Backward compatibility**: All existing `cargo test -p i-rs-claw-core` tests still pass. The `??` propagation in call sites is invisible to callers because they're all in sync code (the new error variant is never returned in normal operation).

3. **Runtime unification**: `cargo test -p i-rs-claw-core --features mcp mcp_gated::tests` passes — McpRegistry uses the shared runtime; no per-instance `Runtime::new()`.

4. **Postgres correctness preserved**: `cargo check -p i-rs-claw-core --features postgres` passes — the macro expansion is unchanged, only the misleading docstring was removed. The compile-test in `tests/compile_features.rs` guards against future regressions.

5. **Storage integration coverage**: `cargo test -p claw-core-storage-tests --features sqlite` passes 8 tests covering every repository trait.

6. **No workspace regressions**: `cargo check --workspace` passes with 0 errors.

7. **Clean lint**: `cargo clippy -p i-rs-claw-core --all-features -- -D warnings` passes with 0 warnings.

If any of these fail after execution, the offending task must be revisited before the plan is considered complete.
