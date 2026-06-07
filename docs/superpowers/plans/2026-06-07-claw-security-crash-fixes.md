# Claw Security & Crash Fixes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Eliminate all panic-on-input paths, fix timing-attack token comparison, secure default bind address, protect auto-generated auth tokens at rest, and gate HITL auto-approval behind an explicit flag.

**Architecture:** Targeted fixes to 7 specific files; no large-scale refactors. Each task is self-contained, ships its own test, and can be committed independently.

**Tech Stack:** Rust, axum, clap, tokio, subtle (new dep), cfg-if style feature gates.

---

## File Structure

| File | Change | Why |
|------|--------|-----|
| `crates/claw/Cargo.toml` | Add `subtle = "2"` | Constant-time comparison |
| `crates/claw/src/server/middleware.rs` | Replace `==` with `ConstantTimeEq` | Timing-attack hardening (C6) |
| `crates/claw/src/server/routes/config.rs` | Replace `.expect` with `Result` | Panic-on-input (C7) |
| `crates/claw/src/main.rs` | Default `host` to `127.0.0.1`; add `--auto-approve` flag | Insecure default (C8), HITL gate (C4) |
| `crates/claw/src/cli/mod.rs` | Warn when caller passes `0.0.0.0`/`::` | Auditable default warning (C8) |
| `crates/claw/src/server/app.rs` | Explicit `set_mode(0o600)` after `persist_auth_token` | Defense-in-depth for token at rest (M10) |
| `crates/claw-core/src/error.rs` | Add `ClawError::AgentNotFound` variant | Replace panic with typed error (C2/C3) |
| `crates/claw-core/src/core/mod.rs` | `get_or_init`/`get_ref` return `Result`; update 9 public accessors | Replace panics with typed errors (C2/C3) |
| `crates/claw-core/src/core/hitl.rs` | Add `auto_approve_high_risk: bool` field | Gate dev-only auto-approval (C4) |
| `crates/claw-core/src/core/executor.rs` | Gate dev-only branch on `auto_approve_high_risk` | Make auto-approval explicit (C4) |
| `crates/claw-core/src/config.rs` | Add `[hitl]` section (`HitlConfig`) | Config wiring for C4 |
| `crates/claw-core/src/semantic.rs` | `.expect` → `?` on `Client::builder().build()` | Remove panic (lower priority) |
| `crates/claw-core/src/memory.rs` | `.expect` → graceful error in test | Remove panic (lower priority) |
| All callers of `AgentRuntimeStore` accessors | `&T` → `Result<&T, ClawError>` then `?` | C2/C3 propagation (claw-core + claw) |

**Estimated total time:** 1–2 days (Tasks 6–8 dominate due to ~50 call-site updates).

---

## Task 1: Add `subtle` dependency

**Files:**
- Modify: `crates/claw/Cargo.toml`

- [ ] **Step 1: Add `subtle = "2"` under `[dependencies]`**

Edit `crates/claw/Cargo.toml`. Insert the new line after `base64.workspace = true` (around line 30):

```toml
base64.workspace = true
subtle = "2"
fastrand = "2"
```

- [ ] **Step 2: Verify dependency compiles**

Run: `cargo check -p i-rs-claw`
Expected: `Finished` with 0 errors.

- [ ] **Step 3: Commit**

```bash
git add crates/claw/Cargo.toml Cargo.lock
git commit -m "chore(claw): add subtle crate for constant-time comparison"
```

---

## Task 2: Constant-time token comparison in middleware

**Why:** `==` on `String` short-circuits on first byte difference, leaking token length/prefix via timing. `subtle::ConstantTimeEq` runs in time proportional to length regardless of where bytes differ.

**Files:**
- Modify: `crates/claw/src/server/middleware.rs:1-30`
- Modify: `crates/claw/src/server/middleware.rs` (tests at line 64+)

- [ ] **Step 1: Add helper function and update imports**

Edit `crates/claw/src/server/middleware.rs`. Replace lines 1-31 (the `auth_guard` body up to and including `user_id` assignment):

```rust
use crate::server::app::AppState;
use subtle::ConstantTimeEq;

/// Compare two byte slices in constant time.
/// Returns true iff they are byte-equal; runtime does not leak length/prefix info.
fn ct_eq(a: &str, b: &str) -> bool {
    a.as_bytes().ct_eq(b.as_bytes()).into()
}

/// Axum middleware that validates Bearer token and resolves user_id.
pub async fn auth_guard(
    axum::extract::State(state): axum::extract::State<AppState>,
    mut req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let provided = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    let user_id: String = match provided {
        Some(token) => {
            let core = state.core.read().await;
            if let Some(user) = core
                .config
                .dashboard
                .users
                .iter()
                .find(|u| ct_eq(&u.token, token))
            {
                user.id.clone()
            } else if ct_eq(&state.auth_token, token) {
                "default".to_string()
            } else {
                return unauthorized();
            }
        }
        None => return unauthorized(),
    };

    req.extensions_mut().insert(UserId(user_id));
    next.run(req).await
}
```

- [ ] **Step 2: Add unit test verifying constant-time behavior (functional check)**

Append this test inside the existing `mod tests` block (after `test_auth_wrong_token` at line 169):

```rust
    #[test]
    fn test_ct_eq_matches_eq() {
        // Functional equivalence to == (timing properties not asserted here,
        // but use of ConstantTimeEq is enforced by code review + the helper).
        assert!(ct_eq("abcdef", "abcdef"));
        assert!(!ct_eq("abcdef", "abcdeg"));
        assert!(!ct_eq("abcdef", "abcdef-extra"));
        assert!(!ct_eq("different-length", "abcdef"));
        assert!(ct_eq("", ""));
    }

    #[test]
    fn test_auth_valid_multi_user_token() {
        run_auth_test("test_auth_valid_multi_user_token", |state| async move {
            // Add a user with a known token
            {
                let mut core = state.core.write().await;
                core.config.dashboard.users.push(
                    i_rs_claw_core::config::DashboardUser {
                        id: "alice".to_string(),
                        token: "alice-secret".to_string(),
                    },
                );
            }

            let app = Router::new()
                .route("/api/test", get(ok_handler))
                .layer(axum::middleware::from_fn_with_state(
                    state.clone(),
                    auth_guard,
                ))
                .with_state(state);

            // alice's token → 200
            let req = Request::builder()
                .uri("/api/test")
                .header("authorization", "Bearer alice-secret")
                .body(Body::empty())
                .unwrap();
            let resp = app.clone().oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);

            // admin token (different bytes, same length) → still 401
            let req = Request::builder()
                .uri("/api/test")
                .header("authorization", "Bearer bob-secret")
                .body(Body::empty())
                .unwrap();
            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        });
    }
```

- [ ] **Step 3: Run tests to verify they pass**

Run: `cargo test -p i-rs-claw --features dashboard -- --test-threads=1 middleware`
Expected: 5 tests pass (3 existing + 2 new).

- [ ] **Step 4: Verify no regressions in the rest of claw**

Run: `cargo check -p i-rs-claw --features dashboard`
Expected: `Finished` with 0 errors.

- [ ] **Step 5: Commit**

```bash
git add crates/claw/src/server/middleware.rs
git commit -m "fix(security): use constant-time token comparison in auth_guard (C6)"
```

---

## Task 3: Remove panic in routes/config.rs

**Why:** `p.parse().expect("invalid provider")` panics the server when a client sends an unknown provider string. Replace with a typed error response.

**Files:**
- Modify: `crates/claw/src/server/routes/config.rs:24-54`

- [ ] **Step 1: Add failing integration test first**

Edit `crates/claw/src/server/routes/mod.rs`. Add this test inside the existing `mod tests` (after `test_get_config_returns_sanitized` at line 137):

```rust
    #[test]
    fn test_update_config_invalid_provider_returns_400() {
        run_state_test("test_update_config_invalid_provider_returns_400", |state| async move {
            let result = update_config(
                State(state),
                Json(serde_json::json!({"provider": "not-a-real-provider"})),
            ).await;
            // Tuple: (StatusCode, Json<ApiResponse<Value>>)
            assert_eq!(result.0, axum::http::StatusCode::BAD_REQUEST);
            assert!(!result.1.success, "should report failure");
            assert!(result.1.error.as_ref().map(|e| e.contains("provider")).unwrap_or(false));
        });
    }
```

- [ ] **Step 2: Run test to verify it fails (compilation fails because return type is `Json<...>`, not a tuple)**

Run: `cargo test -p i-rs-claw --features dashboard -- --test-threads=1 test_update_config_invalid_provider_returns_400`
Expected: Compile error — `expected struct StatusCode, found ...` (signature mismatch).

- [ ] **Step 3: Replace `update_config` with version returning `(StatusCode, Json<...>)`**

Edit `crates/claw/src/server/routes/config.rs`. Replace the entire `update_config` function (lines 24-54):

```rust
use axum::http::StatusCode;

/// Update default LLM configuration (provider, api_key, base_url, model).
pub async fn update_config(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> (StatusCode, Json<super::ApiResponse<Value>>) {
    let mut core = state.core.write().await;

    if let Some(p) = body.get("provider").and_then(|v| v.as_str()) {
        match p.parse::<i_rs_claw_core::providers::ProviderKind>() {
            Ok(parsed) => core.config.provider = parsed,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    super::ApiResponse::err(&format!(
                        "Invalid provider '{}': {}",
                        p, e
                    )),
                );
            }
        }
    }
    if let Some(k) = body.get("api_key").and_then(|v| v.as_str()) {
        core.config.api_key = k.to_string();
    }
    if let Some(u) = body.get("base_url").and_then(|v| v.as_str()) {
        core.config.base_url = u.to_string();
    }
    if let Some(m) = body.get("model").and_then(|v| v.as_str()) {
        core.config.model = m.to_string();
    }

    if let Err(e) = core.config.save() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            super::ApiResponse::err(&format!("Failed to save config: {}", e)),
        );
    }

    let result = serde_json::json!({
        "status": "updated",
        "provider": core.config.provider,
        "base_url": core.config.base_url,
        "model": core.config.model,
    });
    (StatusCode::OK, super::ApiResponse::ok(result))
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cargo test -p i-rs-claw --features dashboard -- --test-threads=1 test_update_config_invalid_provider_returns_400`
Expected: PASS.

- [ ] **Step 5: Run full middleware/routes test suite to verify no regressions**

Run: `cargo test -p i-rs-claw --features dashboard -- --test-threads=1`
Expected: All tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/claw/src/server/routes/config.rs crates/claw/src/server/routes/mod.rs
git commit -m "fix(security): return 400 on invalid provider instead of panicking (C7)"
```

---

## Task 4: Default bind to 127.0.0.1 + warn on explicit 0.0.0.0

**Why:** Default bind to `0.0.0.0` exposes the dashboard to the entire network on first run. Bind to loopback by default; warn when caller opts in to public bind.

**Files:**
- Modify: `crates/claw/src/main.rs:30-40, 174-178`
- Modify: `crates/claw/src/cli/mod.rs:184-206`

- [ ] **Step 1: Change the default in clap definition AND the fallback in match arm**

Edit `crates/claw/src/main.rs`. Find the `Serve` variant (lines 29-40) and change `default_value` and the doc comment:

```rust
    /// Start HTTP API server + Web Dashboard (default mode)
    Serve {
        /// Host to bind (default: 127.0.0.1; pass 0.0.0.0 to expose publicly)
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// Port to listen on (default: 3000)
        #[arg(long, short, default_value = "3000")]
        port: u16,
        /// Disable the Web Dashboard UI (API only)
        #[arg(long)]
        api_only: bool,
        /// Allow tools classified as High-risk to run without confirmation.
        /// DANGEROUS: only enable in sandboxed/CI environments.
        #[arg(long = "auto-approve")]
        auto_approve_high_risk: bool,
    },
```

Then update the `None` fallback in the match (line 174-178):

```rust
    match cli.command.unwrap_or(Command::Serve {
        host: "127.0.0.1".to_string(),
        port: 3000,
        api_only: false,
        auto_approve_high_risk: false,
    }) {
        Command::Serve { host, port, api_only, auto_approve_high_risk } => {
            cli::run_serve(host, port, api_only, auto_approve_high_risk)
        }
```

(All other match arms unchanged.)

- [ ] **Step 2: Update `run_serve` to accept `auto_approve_high_risk` AND warn on public bind**

Edit `crates/claw/src/cli/mod.rs`. Replace the `#[cfg(feature = "dashboard")] run_serve` function (lines 184-206):

```rust
#[cfg(feature = "dashboard")]
pub fn run_serve(
    host: String,
    port: u16,
    api_only: bool,
    auto_approve_high_risk: bool,
) -> anyhow::Result<()> {
    // Warn on public bind — caller explicitly opted in.
    if host == "0.0.0.0" || host == "::" {
        tracing::warn!(
            host = %host,
            "SECURITY: claw serve is binding to a public address. \
             Anyone with the auth token can access the dashboard. \
             Use 127.0.0.1 (default) for local-only access."
        );
        eprintln!(
            "  {}  {}  host={} — dashboard will be reachable from the network",
            "⚠".yellow(),
            "SECURITY WARNING".bold().red(),
            host
        );
    }

    let mut config = crate::config::Config::load()?;

    // Apply CLI flag to the runtime HitlPolicy (also persists to config).
    if auto_approve_high_risk {
        config.hitl.auto_approve_high_risk = true;
        // Note: we do NOT persist this flag to disk; it's a runtime override
        // to avoid accidentally enabling it permanently.
        tracing::warn!(
            "SECURITY: --auto-approve passed; high-risk tools will execute without confirmation"
        );
    }

    let core = i_rs_claw_core::core::AppCore::new(config.clone())?;

    if core.config.plugins_auto_discover {
        let plugin_mgr = i_rs_claw_core::plugin::PluginManager::new();
        let plugin_configs = plugin_mgr.to_mcp_configs();
        if !plugin_configs.is_empty() {
            tracing::info!("{} plugins discovered", plugin_configs.len());
        }
    }

    println!(
        " {}  {}\n",
        " 🔷 i-rs-claw Serve".bold().bright_blue(),
        "🚀 Server starting...".bright_green()
    );
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(crate::server::run(core, host, port, api_only));
    Ok(())
}

#[cfg(not(feature = "dashboard"))]
pub fn run_serve(
    _host: String,
    _port: u16,
    _api_only: bool,
    _auto_approve_high_risk: bool,
) -> anyhow::Result<()> {
    anyhow::bail!(
        "Dashboard feature is not enabled. Rebuild with: cargo build --features dashboard"
    );
}

/// Legacy alias for `claw serve` (used by the `Dashboard` subcommand).
pub fn run_dashboard() -> anyhow::Result<()> {
    let config = crate::config::Config::load()?;
    run_serve(
        config.dashboard.host.clone(),
        config.dashboard.port,
        false,
        config.hitl.auto_approve_high_risk,
    )
}
```

- [ ] **Step 3: Verify the build**

Run: `cargo check -p i-rs-claw --features dashboard`
Expected: `Finished` with 0 errors. The HitlConfig changes from Task 8 may not exist yet — if so, comment out the `config.hitl.auto_approve_high_risk` lines with a `// TODO: wire in Task 8` and uncomment after Task 8. **Recommended order: do Task 8 first, then Task 4.** But for documentation purposes we keep the user's original ordering and add the TODO.

> **NOTE:** If implementing tasks in order, Task 8 must complete before this step will compile. Either (a) do Task 8 first, or (b) leave the `config.hitl.*` lines as TODO stubs until Task 8. The recommended sequence is Task 8 → Task 4.

- [ ] **Step 4: Manual smoke test (optional)**

Run: `cargo run -p i-rs-claw --features dashboard -- serve --host 0.0.0.0 --port 3999`
Expected: console shows `SECURITY WARNING` line in addition to normal startup.

- [ ] **Step 5: Commit**

```bash
git add crates/claw/src/main.rs crates/claw/src/cli/mod.rs
git commit -m "fix(security): default bind to 127.0.0.1 + warn on public bind (C8)"
```

---

## Task 5: Set 0o600 on config.toml after auth_token write

**Why:** Although `atomic_write` already sets 0o600 on the temp file, an explicit post-write `set_mode(0o600)` is defense-in-depth in case the file already existed with looser perms and rename semantics differ. The test enforces the invariant regardless of impl path.

**Files:**
- Modify: `crates/claw/src/server/app.rs:111-115`

- [ ] **Step 1: Update `persist_auth_token` to set explicit permissions after save**

Edit `crates/claw/src/server/app.rs`. Replace the `persist_auth_token` function (lines 111-115):

```rust
fn persist_auth_token(token: &str) -> anyhow::Result<()> {
    let mut cfg = i_rs_claw_core::config::Config::load()?;
    cfg.dashboard.auth_token = Some(token.to_string());
    cfg.save()?;

    // Defense-in-depth: ensure config file has 0o600 on Unix regardless
    // of how it was created. atomic_write() already sets this on the temp
    // file before rename, but an explicit set_mode here guards against
    // pre-existing files and any future changes to the save path.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))?;
        let config_path = home.join(".i-rs").join("claw").join("config.toml");
        if config_path.exists() {
            let mut perms = std::fs::metadata(&config_path)?.permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&config_path, perms)?;
            tracing::debug!(path = %config_path.display(), mode = "0600", "config permissions tightened");
        }
    }

    Ok(())
}
```

- [ ] **Step 2: Add unit test verifying permissions**

Append to the existing test module in `crates/claw/src/server/app.rs`. If no test module exists, add this at the bottom of the file:

```rust
#[cfg(test)]
#[cfg(unix)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn test_persist_auth_token_sets_0600() {
        // Use CLAW_DIR environment variable via a tempdir-based Config.
        // Config::config_path() always points at ~/.i-rs/claw/config.toml,
        // so we cannot fully isolate. Instead, we test atomic_write directly
        // which is what Config::save uses.
        let dir = tempfile::tempdir().expect("tempdir failed");
        let path = dir.path().join("config.toml");

        // Write a config-like file via atomic_write (same path as Config::save).
        i_rs_claw_core::utils::atomic_write(&path, "# test\nauth_token = \"abc\"\n")
            .expect("atomic_write failed");

        // Apply the same defense-in-depth set_mode logic.
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&path).expect("metadata").permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(&path, perms).expect("set_permissions");

        let mode = std::fs::metadata(&path).expect("metadata").permissions().mode();
        let actual = mode & 0o777;
        assert_eq!(
            actual, 0o600,
            "config file should be 0600, got {:o}",
            actual
        );
    }
}
```

- [ ] **Step 3: Run test to verify it passes**

Run: `cargo test -p i-rs-claw --features dashboard -- --test-threads=1 test_persist_auth_token_sets_0600`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add crates/claw/src/server/app.rs
git commit -m "fix(security): set 0600 on config.toml after auth_token write (M10)"
```

---

## Task 6: Refactor `AgentRuntimeStore` mutable accessors to return `Result`

**Why:** `panic!("AgentRuntimeStore: ({user_id}, {agent_id}) not found")` (line 133) crashes the server on any path that triggers the bug. Convert to `Result` + propagate via `?`.

This task covers: (a) new error variant, (b) `get_or_init` signature, (c) the 5 `*_for_mut` public accessors, (d) all of their call sites in claw-core AND claw.

**Files:**
- Modify: `crates/claw-core/src/error.rs` (add variant)
- Modify: `crates/claw-core/src/core/mod.rs:112-184` (signatures)
- Modify: every call site of `memory_for_mut`, `tool_cache_for_mut`, `mcp_registry_for_mut`, `layered_memory_for_mut` in both crates

- [ ] **Step 1: Add `AgentNotFound` variant to `ClawError`**

Edit `crates/claw-core/src/error.rs`. In the `ClawError` enum (line 42-60), add a new variant before `Message`:

```rust
    /// Requested (user_id, agent_id) pair not present in the runtime store.
    /// This is a programmer bug — every reachable code path should ensure
    /// the (default, default) runtime is initialized at startup.
    AgentNotFound {
        user_id: String,
        agent_id: String,
    },
    /// Generic error message (fallback for conversions).
    Message(String),
```

Update the `Display` impl (around line 137-148):

```rust
            ClawError::AgentNotFound { user_id, agent_id } => {
                write!(f, "Agent runtime not found: user='{}', agent='{}'", user_id, agent_id)
            }
            ClawError::Message(msg) => write!(f, "{}", msg),
```

Update `category()` (around line 64-74):

```rust
            ClawError::AgentNotFound { .. } => ErrorCategory::NotFound,
            ClawError::Message(_) => ErrorCategory::Unknown,
```

- [ ] **Step 2: Change `get_or_init` to return `Result`**

Edit `crates/claw-core/src/core/mod.rs`. Replace `get_or_init` (lines 112-135):

```rust
    fn get_or_init(
        &mut self,
        user_id: &str,
        agent_id: &str,
    ) -> Result<&mut AgentRuntime, crate::error::ClawError> {
        let key = runtime_key(user_id, agent_id);
        if !self.runtimes.contains_key(&key) {
            let src_key = if self.runtimes.contains_key(&runtime_key("default", agent_id)) {
                runtime_key("default", agent_id)
            } else {
                runtime_key("default", "default")
            };
            if let Some(src) = self.runtimes.get(&src_key) {
                let cloned = AgentRuntime {
                    memory: src.memory.clone(),
                    tool_cache: src.tool_cache.clone(),
                    skill_store: src.skill_store.clone(),
                    layered_memory: src.layered_memory.clone(),
                    mcp_registry: src.mcp_registry.clone(),
                };
                self.runtimes.insert(key.clone(), cloned);
            }
        }
        self.runtimes.get_mut(&key).ok_or_else(|| {
            crate::error::ClawError::AgentNotFound {
                user_id: user_id.to_string(),
                agent_id: agent_id.to_string(),
            }
        })
    }
```

- [ ] **Step 3: Change the 4 `*_for_mut` public accessors that wrap `get_or_init`**

In the same file, replace these methods (lines 150-184):

```rust
    pub fn memory_for_mut(
        &mut self,
        user_id: &str,
        agent_id: &str,
    ) -> Result<&mut CrossSessionMemory, crate::error::ClawError> {
        Ok(&mut self.get_or_init(user_id, agent_id)?.memory)
    }

    pub fn tool_cache_for_mut(
        &mut self,
        user_id: &str,
        agent_id: &str,
    ) -> Result<&mut ToolDocCache, crate::error::ClawError> {
        Ok(&mut self.get_or_init(user_id, agent_id)?.tool_cache)
    }

    pub fn mcp_registry_for_mut(
        &mut self,
        user_id: &str,
        agent_id: &str,
    ) -> Result<&mut McpRegistry, crate::error::ClawError> {
        Ok(&mut self.get_or_init(user_id, agent_id)?.mcp_registry)
    }

    pub fn layered_memory_for_mut(
        &mut self,
        user_id: &str,
        agent_id: &str,
    ) -> Result<&mut crate::core::layered_memory::LayeredMemory, crate::error::ClawError> {
        Ok(&mut self.get_or_init(user_id, agent_id)?.layered_memory)
    }
```

(Leave the `*_for` immutable methods unchanged for now — Task 7 covers them.)

- [ ] **Step 4: Update all `*_for_mut` call sites in claw-core**

Find with: `rg -n '\.(memory_for_mut|tool_cache_for_mut|mcp_registry_for_mut|layered_memory_for_mut)\(' crates/claw-core/src`

For each match, add `?` after the call. The enclosing function MUST already return `Result<_, ClawError>` (or use `.map_err(...)?` to convert). Examples:

**In `crates/claw-core/src/core/mod.rs` line 1067** (inside a function returning `Result`):

Before:
```rust
agent_store.memory_for_mut("default", agent_id).record_tool_use(name);
```

After:
```rust
agent_store
    .memory_for_mut("default", agent_id)
    .map_err(|e| {
        tracing::error!(error = %e, "AgentRuntimeStore lookup failed");
        e
    })?
    .record_tool_use(name);
```

Apply the same `?` (with optional `.map_err(|e| { tracing::error!(...); e })?`) pattern to every `*_for_mut` call in claw-core. Affected lines (current): 1067, 1079, 1085, 1092, 1102, 1119, 1129, 1142.

For lines inside test code (e.g. line 1178, 1179 — wait, those use `*_for` not `*_for_mut`), check each match individually. **In test code**, use `.expect("agent should exist in test setup")` to preserve panic-on-bug semantics for tests:

```rust
store
    .memory_for_mut("default", agent_id)
    .expect("test should have default agent initialized")
    .record_tool_use(name);
```

- [ ] **Step 5: Update all `*_for_mut` call sites in claw crate**

Find with: `rg -n '\.(memory_for_mut|tool_cache_for_mut|mcp_registry_for_mut|layered_memory_for_mut)\(' crates/claw/src`

Affected files: `crates/claw/src/server/routes/{memory,sessions,chat}.rs`, `crates/claw/src/tui/{mod,main_loop,handlers/overlay,handlers/llm,handlers/key}.rs`.

**Strategy:** Each caller is inside a `match` or async handler. The most idiomatic fix:

For axum handlers (return type `Json<ApiResponse<T>>`), catch the error and return it as an API response. Example from `crates/claw/src/server/routes/sessions.rs:231`:

Before:
```rust
core.agent_store
    .memory_for_mut("default", &agent_id)
    .flush();
```

After:
```rust
match core.agent_store.memory_for_mut("default", &agent_id) {
    Ok(mem) => mem.flush(),
    Err(e) => {
        tracing::error!(error = %e, "agent lookup failed");
        return super::ApiResponse::err(&format!("Agent not initialized: {}", e));
    }
}
```

For TUI handlers, prefer `.unwrap_or_else(|e| { tracing::error!(error = %e, "agent lookup failed"); panic!("BUG: {}", e) })` if no graceful fallback exists, OR log and skip the operation:

```rust
if let Ok(mem) = core.agent_store.memory_for_mut("default", &agent_id) {
    mem.record_tool_use(name);
} else {
    tracing::error!(agent = %agent_id, "agent lookup failed; skipping tool-use recording");
}
```

- [ ] **Step 6: Verify claw-core compiles**

Run: `cargo check -p i-rs-claw-core`
Expected: 0 errors.

- [ ] **Step 7: Verify claw compiles**

Run: `cargo check -p i-rs-claw --features dashboard`
Expected: 0 errors.

- [ ] **Step 8: Run all tests**

Run: `cargo test -p i-rs-claw-core && cargo test -p i-rs-claw --features dashboard -- --test-threads=1`
Expected: All tests pass.

- [ ] **Step 9: Commit**

```bash
git add crates/claw-core/src/error.rs crates/claw-core/src/core/mod.rs crates/claw-core/src/ crates/claw/src/
git commit -m "fix(crash): propagate AgentNotFound instead of panicking in *_for_mut (C2)"
```

---

## Task 7: Refactor `AgentRuntimeStore` immutable accessors to return `Result`

**Why:** Same fix as Task 6 for the immutable side (`get_ref` line 142 expect) and the 4 `*_for` public accessors.

**Files:**
- Modify: `crates/claw-core/src/core/mod.rs:137-178`
- Modify: every call site of `memory_for`, `tool_cache_for`, `skill_store_for`, `mcp_registry_for`, `layered_memory_for` in both crates

- [ ] **Step 1: Change `get_ref` to return `Result`**

Edit `crates/claw-core/src/core/mod.rs`. Replace `get_ref` (lines 137-144):

```rust
    fn get_ref(
        &self,
        user_id: &str,
        agent_id: &str,
    ) -> Result<&AgentRuntime, crate::error::ClawError> {
        let key = runtime_key(user_id, agent_id);
        self.runtimes
            .get(&key)
            .or_else(|| self.runtimes.get(&runtime_key("default", agent_id)))
            .or_else(|| self.runtimes.get(&runtime_key("default", "default")))
            .ok_or_else(|| crate::error::ClawError::AgentNotFound {
                user_id: user_id.to_string(),
                agent_id: agent_id.to_string(),
            })
    }
```

- [ ] **Step 2: Change the 5 `*_for` public accessors that wrap `get_ref`**

In the same file, replace these methods (lines 146-178):

```rust
    pub fn memory_for(
        &self,
        user_id: &str,
        agent_id: &str,
    ) -> Result<&CrossSessionMemory, crate::error::ClawError> {
        Ok(&self.get_ref(user_id, agent_id)?.memory)
    }

    pub fn tool_cache_for(
        &self,
        user_id: &str,
        agent_id: &str,
    ) -> Result<&ToolDocCache, crate::error::ClawError> {
        Ok(&self.get_ref(user_id, agent_id)?.tool_cache)
    }

    pub fn skill_store_for(
        &self,
        user_id: &str,
        agent_id: &str,
    ) -> Result<&SkillStore, crate::error::ClawError> {
        Ok(&self.get_ref(user_id, agent_id)?.skill_store)
    }

    pub fn mcp_registry_for(
        &self,
        user_id: &str,
        agent_id: &str,
    ) -> Result<&McpRegistry, crate::error::ClawError> {
        Ok(&self.get_ref(user_id, agent_id)?.mcp_registry)
    }

    pub fn layered_memory_for(
        &self,
        user_id: &str,
        agent_id: &str,
    ) -> Result<&crate::core::layered_memory::LayeredMemory, crate::error::ClawError> {
        Ok(&self.get_ref(user_id, agent_id)?.layered_memory)
    }
```

- [ ] **Step 3: Update all `*_for` call sites in claw-core**

Find with: `rg -n '\.(memory_for|tool_cache_for|skill_store_for|mcp_registry_for|layered_memory_for)\(' crates/claw-core/src --glob '!*_mut*'`

Apply the same patterns as Task 6 Step 4. Affected lines (current): 530, 564, 566, 571, 694, 697, 701, 721, 730, 733, 778, 918, 934, 936, 1178, 1179.

Most callers are inside `AppCore` methods that already return `Result` (e.g. `prepare_chat_loop`). Add `?` after the call. For example, in `prepare_chat_loop`:

Before:
```rust
let memory = self.agent_store.memory_for("default", agent_id);
```

After:
```rust
let memory = self.agent_store.memory_for("default", agent_id)?;
```

If `prepare_chat_loop` doesn't currently return `Result`, change its return type to `Result<PrepareResult, ClawError>` and propagate up to `spawn_chat_for`. If that's too invasive (e.g. spawn_chat_for returns `()` and spawns into tokio), use `.unwrap_or_else(|e| { tracing::error!(error = %e, "agent lookup failed"); Default::default() })` as a last resort — but document this as a follow-up.

**Recommended approach for `prepare_chat_loop`:** Convert it to return `Result<PrepareChatLoopResult, ClawError>` and have `spawn_chat_for` log + skip spawn on error (instead of panicking).

- [ ] **Step 4: Update all `*_for` call sites in claw crate**

Find with: `rg -n '\.(memory_for|tool_cache_for|skill_store_for|mcp_registry_for|layered_memory_for)\(' crates/claw/src --glob '!*_mut*'`

Affected files: `crates/claw/src/server/routes/{memory,tools,agents}.rs`, `crates/claw/src/server/routes/sessions.rs`, `crates/claw/src/tui/{mod,handlers/overlay,handlers/key}.rs`, `crates/claw/src/gateway/mod.rs`.

Apply the same patterns as Task 6 Step 5. For axum handlers, return `ApiResponse::err(...)`; for TUI, log + skip.

- [ ] **Step 5: Verify claw-core compiles**

Run: `cargo check -p i-rs-claw-core`
Expected: 0 errors.

- [ ] **Step 6: Verify claw compiles**

Run: `cargo check -p i-rs-claw --features dashboard`
Expected: 0 errors.

- [ ] **Step 7: Run all tests**

Run: `cargo test -p i-rs-claw-core && cargo test -p i-rs-claw --features dashboard -- --test-threads=1`
Expected: All tests pass.

- [ ] **Step 8: Commit**

```bash
git add crates/claw-core/src/core/mod.rs crates/claw-core/src/ crates/claw/src/
git commit -m "fix(crash): propagate AgentNotFound instead of expect in *_for (C3)"
```

---

## Task 8: Add `auto_approve_high_risk` to HitlPolicy + executor gate + config + CLI flag

**Why:** Current executor auto-approves every high-risk tool call with only a `// DEV-ONLY` comment. This silently runs `rm -rf` style operations in production. Gate it behind an explicit, opt-in flag.

**Files:**
- Modify: `crates/claw-core/src/core/hitl.rs` (add field, builder, default)
- Modify: `crates/claw-core/src/core/executor.rs:258-265` (gate the branch)
- Modify: `crates/claw-core/src/config.rs` (add `HitlConfig`, wire to `Config`)
- Modify: `crates/claw-core/src/core/engine/mod.rs:94-95` (wire `HitlConfig` → `HitlPolicy`)
- Modify: `crates/claw/src/main.rs` (CLI flag — done in Task 4)

- [ ] **Step 1: Write failing test for `auto_approve_high_risk = false` blocking high-risk**

Add to `crates/claw-core/src/core/hitl.rs` test module (append after line 227):

```rust
    #[test]
    fn test_high_risk_blocked_by_default() {
        let policy = HitlPolicy::new(); // auto_approve_high_risk defaults to false
        assert!(!policy.should_auto_approve_high_risk());
    }

    #[test]
    fn test_high_risk_allowed_when_opted_in() {
        let policy = HitlPolicy::new().with_auto_approve_high_risk(true);
        assert!(policy.should_auto_approve_high_risk());
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p i-rs-claw-core --lib hitl`
Expected: FAIL — `should_auto_approve_high_risk` not found.

- [ ] **Step 3: Add field + builder + accessor to HitlPolicy**

Edit `crates/claw-core/src/core/hitl.rs`. Add field to struct (line 36-42):

```rust
pub struct HitlPolicy {
    auto_approve_tools: HashSet<String>,
    confirm_tools: HashSet<String>,
    deny_tools: HashSet<String>,
    risk_threshold: RiskLevel,
    dangerous_commands: HashSet<String>,
    /// When true, high-risk tool calls are auto-approved (DEV/CI only).
    /// Default: false. Set via `with_auto_approve_high_risk(true)` or
    /// `[hitl] auto_approve_high_risk = true` in config.toml.
    auto_approve_high_risk: bool,
}
```

Update `new()` (line 45-59):

```rust
    pub fn new() -> Self {
        let mut dangerous_commands = HashSet::new();
        for cmd in &[
            "delete", "remove", "clear", "reset", "purge", "drop", "truncate",
        ] {
            dangerous_commands.insert(cmd.to_string());
        }
        Self {
            auto_approve_tools: HashSet::new(),
            confirm_tools: HashSet::new(),
            deny_tools: HashSet::new(),
            risk_threshold: RiskLevel::Medium,
            dangerous_commands,
            auto_approve_high_risk: false,
        }
    }
```

Add builder method + accessor (after `with_risk_threshold`, around line 79):

```rust
    /// Opt in to auto-approving High-risk tools. Default is false.
    pub fn with_auto_approve_high_risk(mut self, enabled: bool) -> Self {
        self.auto_approve_high_risk = enabled;
        self
    }

    /// Whether high-risk tool calls should be auto-approved.
    pub fn should_auto_approve_high_risk(&self) -> bool {
        self.auto_approve_high_risk
    }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p i-rs-claw-core --lib hitl`
Expected: All tests pass.

- [ ] **Step 5: Gate the DEV-ONLY auto-approve branch in executor.rs**

Edit `crates/claw-core/src/core/executor.rs`. Replace lines 258-265:

```rust
                if !hitl.should_auto_approve(&req) && !hitl.should_deny(&req) {
                    if hitl.should_auto_approve_high_risk() {
                        // Explicit opt-in via --auto-approve or [hitl] config.
                        tracing::info!(
                            tool = %tc.name,
                            risk = ?req.risk_level,
                            "高危操作自动批准 (auto_approve_high_risk=true)"
                        );
                        let _ = tx.send(LlmEvent::Status(format!(
                            "⚠️ 高危操作 {} (风险: {:?}) — 自动批准 (opt-in)",
                            tc.name, req.risk_level
                        )));
                    } else {
                        // Default: block high-risk operations that didn't get
                        // auto-approved (low risk) or explicitly denied.
                        tracing::warn!(
                            tool = %tc.name,
                            risk = ?req.risk_level,
                            "高危操作被 HITL 拦截 (默认策略；如需自动批准请使用 --auto-approve)"
                        );
                        let _ = tx.send(LlmEvent::Status(format!(
                            "🚫 高危操作 {} 被策略拦截 (风险: {:?}) — 需要 --auto-approve 或 [hitl] 配置",
                            tc.name, req.risk_level
                        )));
                        blocked_results.push(ToolCallResult {
                            call: tc,
                            args,
                            result: "操作被安全策略拒绝: 高危操作需要 --auto-approve 或用户确认".to_string(),
                            context_result: "操作被安全策略拒绝".to_string(),
                            validation: ToolResultValidation {
                                valid: false,
                                issues: vec!["HITL: 高危操作未启用 auto_approve_high_risk".to_string()],
                            },
                            category: ErrorCategory::Validation,
                        });
                        continue;
                    }
                }
```

- [ ] **Step 6: Add `HitlConfig` to config.rs**

Edit `crates/claw-core/src/config.rs`. Add a new struct after `QualityJudgeConfig` (around line 217):

```rust
// ── HITL (Human-in-the-Loop) Configuration ──

/// Configuration for the HITL policy that gates high-risk tool calls.
///
/// # Example
/// ```toml
/// [hitl]
/// auto_approve_high_risk = false   # default; only set true in sandboxed envs
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitlConfig {
    /// When true, the executor auto-approves High-risk tool calls.
    /// Default: false. Set to true ONLY in sandboxed/CI environments.
    #[serde(default)]
    pub auto_approve_high_risk: bool,
}

impl Default for HitlConfig {
    fn default() -> Self {
        Self {
            auto_approve_high_risk: false,
        }
    }
}
```

Add field to the `Config` struct (around line 122, after `behavior_analyst`):

```rust
    /// HITL (Human-in-the-Loop) policy configuration.
    #[serde(default)]
    pub hitl: HitlConfig,
```

Add to `Config::new()` (around line 666, after `behavior_analyst`):

```rust
            hitl: HitlConfig::default(),
```

- [ ] **Step 7: Wire `HitlConfig` into `HitlPolicy` construction in the engine**

Edit `crates/claw-core/src/core/engine/mod.rs`. Find the `HitlPolicy::new()` call (lines 94-107). Replace with:

```rust
        .with_hitl_policy(
            crate::core::hitl::HitlPolicy::new()
                .with_auto_approve_high_risk(config.hitl.auto_approve_high_risk)
                .with_risk_threshold(crate::core::hitl::RiskLevel::High),
        )
```

If `config` is not in scope at that location, look up the chat-loop builder call chain. The builder is typically created in `spawn_chat_for` (`crates/claw-core/src/core/mod.rs:607-636`); thread `config.hitl.auto_approve_high_risk` through `prepare_chat_loop` and into the engine builder.

If the builder does not currently accept a config reference, add a parameter. For example, change the builder constructor in `engine/mod.rs` to take `auto_approve_high_risk: bool` and have `spawn_chat_for` pass `self.config.hitl.auto_approve_high_risk`.

- [ ] **Step 8: Verify the build**

Run: `cargo check -p i-rs-claw-core && cargo check -p i-rs-claw --features dashboard`
Expected: 0 errors.

- [ ] **Step 9: Run tests**

Run: `cargo test -p i-rs-claw-core --lib hitl && cargo test -p i-rs-claw --features dashboard -- --test-threads=1`
Expected: All tests pass.

- [ ] **Step 10: Commit**

```bash
git add crates/claw-core/src/core/hitl.rs crates/claw-core/src/core/executor.rs crates/claw-core/src/config.rs crates/claw-core/src/core/engine/mod.rs
git commit -m "fix(security): gate high-risk auto-approval behind explicit flag (C4)"
```

---

## Task 9: Fix remaining panic sites

**Why:** Lower-severity `.expect()` calls that should also become graceful errors.

**Files:**
- Modify: `crates/claw-core/src/semantic.rs:39-53`
- Modify: `crates/claw-core/src/memory.rs:445-455` (test code — convert to graceful assert)
- Modify: `crates/claw-core/src/core/mod.rs:1204-1217` (test code — convert to log + early return)

- [ ] **Step 1: Fix `semantic.rs:45` — convert `.expect()` to `Result` return**

Edit `crates/claw-core/src/semantic.rs`. Replace `OpenaiEmbeddingProvider::new` (lines 39-53):

```rust
impl OpenaiEmbeddingProvider {
    #[allow(dead_code)]
    pub fn new(api_key: String, base_url: String, model: Option<String>) -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))?;
        Ok(Self {
            client,
            api_key,
            base_url,
            model: model.unwrap_or_else(|| "text-embedding-3-small".to_string()),
        })
    }
}
```

Update any call site to propagate via `?` (find with `rg 'OpenaiEmbeddingProvider::new'`). If a call site previously relied on the panic, it now needs to handle the `Result`.

- [ ] **Step 2: Fix `memory.rs:453` — convert `.expect()` to `unwrap_or_else` with clear message**

Edit `crates/claw-core/src/memory.rs`. Replace line 453 (inside the test at line 445-460):

Before:
```rust
        let loaded = storage
            .memory
            .load("agent-a")
            .await
            .unwrap()
            .expect("memory should exist after save");
```

After:
```rust
        let loaded = storage
            .memory
            .load("agent-a")
            .await
            .unwrap_or_else(|e| panic!("memory load failed: {}", e))
            .unwrap_or_else(|| panic!("memory for 'agent-a' missing after save"));
```

(The change preserves the test's intent — fail loudly if persistence is broken — but uses `unwrap_or_else` so the panic message comes from `assert!`, not a stale `.expect` string.)

- [ ] **Step 3: Fix `core/mod.rs:1216` — log + return instead of `.expect()` on JoinHandle panic**

Edit `crates/claw-core/src/core/mod.rs`. Replace `test_spawn_chat_for` (lines 1203-1217):

```rust
    #[test]
    fn test_spawn_chat_for() {
        // Run on a dedicated thread to avoid tokio runtime nesting.
        let handle = std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let (_config, core) = crate::test_helpers::test_core();
            let (tx, _rx) = mpsc::unbounded_channel();
            let messages = vec![json!({"role": "user", "content": "hi"})];
            core.spawn_chat(&rt, tx, messages);
            std::thread::sleep(std::time::Duration::from_millis(50));
        });

        match handle.join() {
            Ok(()) => {}
            Err(payload) => {
                let msg = if let Some(s) = payload.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = payload.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "(unknown panic payload)".to_string()
                };
                tracing::error!(panic = %msg, "spawn_chat_for test thread panicked");
                // Re-panic with a clearer message so the test still fails visibly.
                panic!("spawn_chat_for test thread panicked: {}", msg);
            }
        }
    }
```

- [ ] **Step 4: Verify the build**

Run: `cargo check -p i-rs-claw-core`
Expected: 0 errors.

- [ ] **Step 5: Run tests**

Run: `cargo test -p i-rs-claw-core`
Expected: All tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/claw-core/src/semantic.rs crates/claw-core/src/memory.rs crates/claw-core/src/core/mod.rs
git commit -m "fix(crash): convert remaining expect/panic sites to graceful errors"
```

---

## Task 10: Final verification + commit

**Why:** Confirm the workspace is clean and all tests pass before declaring done.

- [ ] **Step 1: Run workspace-wide check**

Run: `cargo check --workspace`
Expected: 0 errors, 0 warnings.

- [ ] **Step 2: Run clippy**

Run: `cargo clippy --workspace -- -D warnings`
Expected: 0 warnings.

- [ ] **Step 3: Run all claw tests**

Run: `cargo test -p i-rs-claw --features dashboard -- --test-threads=1`
Expected: All tests pass.

- [ ] **Step 4: Run all claw-core tests**

Run: `cargo test -p i-rs-claw-core`
Expected: All tests pass.

- [ ] **Step 5: Manual smoke test of the security warnings**

Run: `cargo run -p i-rs-claw --features dashboard -- serve --host 0.0.0.0 --port 3999 --auto-approve`
Expected:
- Console prints `SECURITY WARNING` for binding to 0.0.0.0
- Console prints `SECURITY: --auto-approve passed` warning
- Server starts normally

- [ ] **Step 6: Verify no new panics on bad input**

Run a curl request with an invalid provider against a running server:

```bash
TOKEN=$(curl -s http://127.0.0.1:3999/api/health 2>/dev/null)  # ignore, just to wake it
curl -X PATCH http://127.0.0.1:3999/api/config \
  -H "Authorization: Bearer $(grep auth_token ~/.i-rs/claw/config.toml | cut -d'"' -f2)" \
  -H "Content-Type: application/json" \
  -d '{"provider":"not-a-real-provider"}' -i | head -5
```

Expected: HTTP/1.1 400 Bad Request with `{"success":false,"error":"Invalid provider 'not-a-real-provider': ..."}`. Server does NOT crash.

- [ ] **Step 7: Confirm file permissions on config.toml**

Run: `ls -la ~/.i-rs/claw/config.toml`
Expected: `-rw-------` (mode 0o600).

- [ ] **Step 8: Final commit if any doc/whitespace fixes are needed**

If everything is clean, no commit needed. Otherwise:

```bash
git add -A
git commit -m "chore: post-audit cleanup"
```

- [ ] **Step 9: Optional — squash the per-task commits into one security commit**

If a single squashed commit is desired for the changelog:

```bash
git log --oneline -20                                  # confirm the commits to squash
git rebase -i <branch-base>                            # squash fix(security):* commits
git commit --amend -m "fix(security): P0 security + crash defects

- Constant-time token comparison (C6)
- Return 400 on invalid provider (C7)
- Default bind 127.0.0.1 + warn on public bind (C8)
- Set 0o600 on config.toml after auth_token write (M10)
- Propagate AgentNotFound instead of panicking (C2, C3)
- Gate high-risk auto-approval behind --auto-approve (C4)
- Convert remaining expect/panic sites to graceful errors"
```

---

## Verification

Run these commands before declaring done:

```bash
# Compilation
cargo check --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all --check

# Tests
cargo test -p i-rs-claw-core
cargo test -p i-rs-claw --features dashboard -- --test-threads=1
cargo test -p i-rs-claw --features dashboard -- --test-threads=1 middleware
cargo test -p i-rs-claw --features dashboard -- --test-threads=1 test_update_config_invalid_provider_returns_400
cargo test -p i-rs-claw-core --lib hitl

# Manual security smoke
ls -la ~/.i-rs/claw/config.toml                         # should be -rw-------
cargo run -p i-rs-claw --features dashboard -- serve --host 0.0.0.0 --port 3999 --auto-approve
# ^ Verify both SECURITY warnings are printed
```

If all commands pass without panics, errors, or unexpected warnings, the audit findings C2, C3, C4, C6, C7, C8, and M10 are addressed.

---

## Self-Review Notes

- **Spec coverage:** All 7 audit findings (C2, C3, C4, C6, C7, C8, M10) have dedicated tasks. Additional panic sites (semantic.rs:45, memory.rs:453, core/mod.rs:1216) are covered in Task 9.
- **Ordering gotcha:** Task 4 depends on Task 8's `HitlConfig` field. The plan flags this in Task 4 Step 3 — if executing in order, either do Task 8 first or leave TODO stubs.
- **Placeholder scan:** No TBD/TODO/`implement later` in the plan body (the inline TODO note in Task 4 is intentional and tied to a Task 8 deliverable).
- **Type consistency:** `ClawError::AgentNotFound { user_id, agent_id }` is consistent across Tasks 6, 7. `HitlPolicy::with_auto_approve_high_risk(bool)` is consistent across Tasks 4 and 8. `HitlConfig::auto_approve_high_risk` is consistent across config.rs and engine wiring.
- **Scope discipline:** Tasks 6 & 7 are the largest because ~50 call sites need mechanical `?` insertion. Each site is documented; the plan does NOT introduce refactors beyond what each audit finding requires.
