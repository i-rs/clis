# 存储层测试计划 Implementation Plan

&gt; **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a multi-layer test suite covering 6 storage drivers (File/SQLite/MySQL/PG/Mongo/Redis) with contract tests, CLI-based conversation scripts, and actionable test reports.

**Architecture:** Two layers — (1) contract tests directly exercise ClawStorage trait against each backend via a shared macro; (2) conversation scripts run guided CLI tutorials through the agent, verifying storage state after each step. A reporter module generates Markdown+JSON output with intent analysis, token tracking, and system prompt evolution.

**Tech Stack:** Rust, tokio test, SQLite (rusqlite), MySQL (sqlx), PostgreSQL (sqlx), docker-compose, tempfile, serde_json

**Base crate:** `crates/claw-core-storage-tests/` (extends existing crate, 8 SQLite tests already present)

---

## Phase 0: Foundation — Contract Tests (File + SQLite)

### Task 1: Setup crate dependencies and lib.rs

**Files:**
- Modify: `crates/claw-core-storage-tests/Cargo.toml`
- Modify: `crates/claw-core-storage-tests/src/lib.rs`
- Create: `crates/claw-core-storage-tests/src/contracts/mod.rs`

**Step 1.1: Update Cargo.toml**

Add `tempfile` and enable `sqlite` + `mysql` + `postgres` features for the test crate:

```toml
[package]
name = "claw-core-storage-tests"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Integration tests for i-rs-claw-core storage backends"

[dev-dependencies]
i-rs-claw-core = { path = "../claw-core", features = ["sqlite", "mysql", "postgres"] }
chrono.workspace = true
tokio.workspace = true
serde_json.workspace = true
anyhow.workspace = true
uuid.workspace = true
tempfile.workspace = true
```

- [ ] **Commit:** `git add crates/claw-core-storage-tests/Cargo.toml && git commit -m "chore: add tempfile dep to storage-tests"`

**Step 1.2: Create contracts/mod.rs with the contract! macro**

```rust
// crates/claw-core-storage-tests/src/contracts/mod.rs

/// Macro to define a storage contract test.
///
/// Usage:
/// ```ignore
/// contract!("SessionRepo", |storage: &ClawStorage| {
///     // test assertions here
/// });
/// ```
///
/// The macro generates a function with signature:
/// `pub async fn run_session_contract(storage: &ClawStorage) -> Result<()>`
#[macro_export]
macro_rules! contract {
    ($name:expr, $body:expr) => {
        pub async fn run_contract(storage: &crate::ClawStorage) -> anyhow::Result<()> {
            let test_fn: fn(&crate::ClawStorage) -> BoxFuture<anyhow::Result<()>> =
                Box::new(move |s| Box::pin(async move { ($body)(s).await }));
            test_fn(storage).await
        }
    };
}
```

Wait — simpler approach: just define a trait or use plain functions. Each contract file exports a single async function that takes `&ClawStorage` and returns `anyhow::Result<()>`.

```rust
// contracts/mod.rs
pub mod session_contract;
pub mod message_log_contract;
pub mod memory_contract;
pub mod stats_contract;
pub mod config_store_contract;
pub mod tool_cache_contract;
pub mod skill_contract;
```

- [ ] **Commit:** `git add crates/claw-core-storage-tests/src/contracts/mod.rs && git commit -m "feat: add contracts module tree"`

**Step 1.3: Create define_backend_tests! macro**

Create `crates/claw-core-storage-tests/src/backends/mod.rs`:

```rust
// crates/claw-core-storage-tests/src/backends/mod.rs

/// Macro: define a test module that runs all contracts against a backend.
///
/// Usage:
/// ```ignore
/// define_backend_tests!("sqlite", || async {
///     let tmp = tempfile::tempdir().unwrap();
///     let config = StorageConfig { backend: StorageBackend::Sqlite, sqlite_path: Some(...), .. };
///     ClawStorage::sqlite(&config).await.unwrap()
/// });
/// ```
#[macro_export]
macro_rules! define_backend_tests {
    ($name:expr, $factory:expr) => {
        #[cfg(test)]
        mod ${concat($name, "_contracts")} {
            use super::*;
            use crate::contracts::*;

            fn setup_backend() -> std::sync::Arc<ClawStorage> {
                std::sync::Arc::new(tokio::runtime::Runtime::new().unwrap().block_on($factory))
            }

            #[tokio::test]
            async fn session_contract() -> anyhow::Result<()> {
                let storage = setup_backend();
                session_contract::run(&storage).await
            }

            #[tokio::test]
            async fn message_log_contract() -> anyhow::Result<()> {
                let storage = setup_backend();
                message_log_contract::run(&storage).await
            }

            // ... one per contract
        }
    };
}
```

Simplify — instead of a macro, just write explicit test modules per backend. The macro adds abstraction cost for little gain. Use a shared helper function instead.

**Decision:** Use a `run_all_contracts(storage: Arc<ClawStorage>)` function that each backend test module calls:

```rust
// backends/mod.rs
pub mod helpers;

/// Run all 7 contracts against a storage backend.
/// Returns a list of (contract_name, Result) for reporting.
pub async fn run_all_contracts(
    storage: Arc<ClawStorage>,
) -> Vec<(&'static str, anyhow::Result<()>)> {
    let mut results = Vec::new();
    results.push(("session", contracts::session::run(&storage).await));
    results.push(("message_log", contracts::message_log::run(&storage).await));
    results.push(("memory", contracts::memory::run(&storage).await));
    results.push(("stats", contracts::stats::run(&storage).await));
    results.push(("config_store", contracts::config_store::run(&storage).await));
    results.push(("tool_cache", contracts::tool_cache::run(&storage).await));
    results.push(("skill", contracts::skill::run(&storage).await));
    results
}
```

- [ ] **Commit:** `git add crates/claw-core-storage-tests/src/backends/mod.rs && git commit -m "feat: add run_all_contracts helper"`

---

### Task 2: Session Contract

**File:** Create `crates/claw-core-storage-tests/src/contracts/session.rs`

```rust
use i_rs_claw_core::ClawStorage;
use i_rs_claw_core::session::SessionMeta;
use i_rs_claw_core::session::SessionState;

pub async fn run(storage: &ClawStorage) -> anyhow::Result<()> {
    // 1. Upsert a session
    let meta = SessionMeta {
        id: "test-session-1".into(),
        title: "Test".into(),
        agent_id: "default".into(),
        user_id: "default".into(),
        state: SessionState::Active,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        message_count: 0,
    };
    storage.sessions.upsert(&meta).await?;

    // 2. Load all — should have at least 1
    let all = storage.sessions.load_all().await?;
    assert!(all.len() >= 1, "expected at least 1 session");

    // 3. Get one by ID
    let loaded = storage.sessions.get_one("test-session-1").await?;
    assert!(loaded.is_some());
    assert_eq!(loaded.unwrap().title, "Test");

    // 4. Update message_count
    let mut updated = meta.clone();
    updated.message_count = 5;
    updated.state = SessionState::Completed;
    storage.sessions.upsert(&updated).await?;
    let reloaded = storage.sessions.get_one("test-session-1").await?.unwrap();
    assert_eq!(reloaded.message_count, 5);
    assert_eq!(reloaded.state, SessionState::Completed);

    // 5. Count
    let count = storage.sessions.count().await?;
    assert!(count >= 1);

    // 6. Delete
    storage.sessions.delete_one("test-session-1").await?;
    let after_delete = storage.sessions.get_one("test-session-1").await?;
    assert!(after_delete.is_none());

    Ok(())
}
```

- [ ] **Step: Write the contract**
- [ ] **Step: Verify compile:** `cargo check -p claw-core-storage-tests 2>&1 | grep error | wc -l` → 0
- [ ] **Step: Commit**

---

### Task 3: MessageLog Contract

**File:** Create `crates/claw-core-storage-tests/src/contracts/message_log.rs`

```rust
use i_rs_claw_core::storage::ClawStorage;
use i_rs_claw_core::app::Message;
use i_rs_claw_core::app::MessageRole;

fn make_test_msgs(n: usize) -> Vec<Message> {
    (0..n).map(|i| Message {
        role: MessageRole::User,
        content: Some(format!("test message {}", i)),
        tool_calls: None,
        tool_call_id: None,
        name: None,
    }).collect()
}

pub async fn run(storage: &ClawStorage) -> anyhow::Result<()> {
    let sid = "msg-test-session";

    // 1. Append batch
    let msgs = make_test_msgs(3);
    storage.message_log.append_batch(sid, &msgs).await?;

    // 2. Load all
    let loaded = storage.message_log.load(sid, 100).await?;
    assert_eq!(loaded.len(), 3);

    // 3. Load with limit
    let limited = storage.message_log.load(sid, 2).await?;
    assert_eq!(limited.len(), 2);

    // 4. Append more
    let more = make_test_msgs(2);
    storage.message_log.append_batch(sid, &more).await?;
    let all = storage.message_log.load(sid, 100).await?;
    assert_eq!(all.len(), 5);

    // 5. Search (case-insensitive substring)
    let found = storage.message_log.search("TEST MESSAGE", 10).await?;
    assert!(!found.is_empty(), "search should find results");

    // 6. Count
    let count = storage.message_log.count(sid).await?;
    assert_eq!(count, 5);

    // 7. Delete session
    storage.message_log.delete_session(sid).await?;
    let after = storage.message_log.load(sid, 100).await?;
    assert!(after.is_empty());

    Ok(())
}
```

- [ ] **Step: Write the contract**
- [ ] **Step: Verify compile**
- [ ] **Step: Commit**

---

### Task 4: Memory Contract

**File:** Create `crates/claw-core-storage-tests/src/contracts/memory.rs`

Tests `MemoryRepo::save()` and `MemoryRepo::load()`. Memory is a `CrossSessionMemory` serialized as JSON.

```rust
use i_rs_claw_core::ClawStorage;
use i_rs_claw_core::memory::CrossSessionMemory;

pub async fn run(storage: &ClawStorage) -> anyhow::Result<()> {
    let agent = "memory-test-agent";

    // 1. Save memory with tool frequency
    let mut mem = CrossSessionMemory::for_agent(storage.clone(), agent.into());
    mem.record_tool_use("i-rs-weight");
    mem.record_tool_use("i-rs-weight");
    mem.record_tool_use("i-rs-todo");
    mem.set_user_name("测试用户");
    mem.flush().await?;

    // 2. Load — should persist tool_frequency
    let loaded = CrossSessionMemory::for_agent(storage.clone(), agent.into());
    // Use format_user_memory to verify content
    let formatted = loaded.format_user_memory();
    assert!(formatted.contains("i-rs-weight"), "memory should contain tool frequency");

    // 3. Update and re-save
    loaded.record_tool_use("i-rs-weight");
    loaded.flush().await?;

    // 4. Verify update persisted
    let reloaded = CrossSessionMemory::for_agent(storage.clone(), agent.into());
    assert!(reloaded.format_user_memory().contains("3"), "weight count should be 3");

    // 5. Cleanup: save empty
    let empty = CrossSessionMemory::for_agent(storage.clone(), agent.into());
    empty.flush().await?;

    Ok(())
}
```

Note: `CrossSessionMemory::for_agent()` takes `Arc<ClawStorage>`, not `&ClawStorage`. We may need to adjust the contract signature or clone the Arc.

- [ ] **Step: Write the contract**
- [ ] **Step: Verify compile**
- [ ] **Step: Commit**

---

### Task 5: Stats Contract

**File:** Create `crates/claw-core-storage-tests/src/contracts/stats.rs`

Tests `StatsRepo::upsert_batch()`, `read_range()`, `prune()`.

```rust
use i_rs_claw_core::ClawStorage;
use i_rs_claw_core::stats::TokenRecord;
use chrono::{Utc, Duration};

pub async fn run(storage: &ClawStorage) -> anyhow::Result<()> {
    let now = Utc::now();

    // 1. Insert batch records
    let records: Vec<TokenRecord> = (0..5).map(|i| TokenRecord {
        id: format!("stat-{}", i),
        timestamp: now - Duration::hours(i as i64 * 24),
        user_id: "default".into(),
        agent_id: "default".into(),
        model: "gpt-4".into(),
        provider: "openai".into(),
        prompt_tokens: 100 + i * 10,
        completion_tokens: 50 + i * 5,
        total_tokens: 150 + i * 15,
        has_tool_calls: i % 2 == 0,
        tool_call_count: if i % 2 == 0 { 2 } else { 0 },
        react_rounds: 1,
        success: true,
        latency_ms: 500 + i * 100,
        estimated_cost_usd: 0.001 * (i as f64 + 1.0),
        trace_id: Some(format!("trace-{}", i)),
    }).collect();
    storage.stats.upsert_batch(&records).await?;

    // 2. Read range (last 30 days)
    let all = storage.stats.read_range(
        now - Duration::days(30),
        now + Duration::days(1),
    ).await?;
    assert_eq!(all.len(), 5, "should have 5 records in range");

    // 3. Read range (last 2 days only — should get stat-0 and maybe stat-1)
    let recent = storage.stats.read_range(
        now - Duration::days(2),
        now + Duration::days(1),
    ).await?;
    assert!(recent.len() >= 1, "should have at least 1 recent record");

    // 4. Prune (keep 3 days — deletes records older than 3 days)
    storage.stats.prune(3).await?;
    let after_prune = storage.stats.read_range(
        Utc::now() - Duration::days(365),
        Utc::now() + Duration::days(1),
    ).await?;
    // Should have fewer records now
    assert!(after_prune.len() < 5, "prune should remove old records");

    Ok(())
}
```

- [ ] **Step: Write the contract**
- [ ] **Step: Verify compile**
- [ ] **Step: Commit**

---

### Task 6: ConfigStore Contract

**File:** Create `crates/claw-core-storage-tests/src/contracts/config_store.rs`

Tests all 5 ConfigStore repos: agent_configs, provider_configs, dashboard_users, mcp_servers, app_settings.

```rust
use i_rs_claw_core::{ConfigStore, AgentConfig, ProviderConfig};
use std::collections::HashMap;

pub async fn run(storage: &ConfigStore) -> anyhow::Result<()> {
    // 1. Agent config CRUD
    let mut config = AgentConfig::default();
    config.system_prompt = Some("You are a test".into());
    storage.agent_configs.save("test-agent", &config).await?;
    let loaded = storage.agent_configs.load("test-agent").await?;
    assert!(loaded.is_some());
    assert_eq!(loaded.unwrap().system_prompt.unwrap(), "You are a test");
    storage.agent_configs.delete("test-agent").await?;
    let after = storage.agent_configs.load("test-agent").await?;
    assert!(after.is_none());

    // 2. Provider config CRUD
    let provider = ProviderConfig {
        provider: "openai".into(),
        api_key: Some("sk-test".into()),
        base_url: None,
        model: Some("gpt-4".into()),
        ..Default::default()
    };
    storage.provider_configs.save("test-provider", &provider).await?;
    let loaded_p = storage.provider_configs.load("test-provider").await?;
    assert_eq!(loaded_p.unwrap().api_key.unwrap(), "sk-test");
    storage.provider_configs.delete("test-provider").await?;

    // 3. Dashboard users CRUD
    let users = vec!["user1".into(), "user2".into()];
    storage.dashboard_users.save_all(&users).await?;
    let loaded_users = storage.dashboard_users.load_all().await?;
    assert_eq!(loaded_users.len(), 2);
    storage.dashboard_users.save_all(&[] as &Vec<String>).await?;

    // 4. MCP server config CRUD
    let mcp = i_rs_claw_core::McpServerConfig {
        name: "test-mcp".into(),
        transport_type: "stdio".into(),
        command: "echo".into(),
        args: Some(vec!["hello".into()]),
        env: None,
    };
    storage.mcp_servers.save(&mcp).await?;
    let loaded_m = storage.mcp_servers.load("test-mcp").await?;
    assert!(loaded_m.is_some());
    storage.mcp_servers.delete("test-mcp").await?;

    // 5. App settings CRUD
    let settings: HashMap<String, String> = [("theme".into(), "dark".into())].into();
    storage.app_settings.save(&settings).await?;
    let loaded_s = storage.app_settings.load().await?;
    assert_eq!(loaded_s.get("theme").unwrap(), "dark");

    Ok(())
}
```

- [ ] **Step: Write the contract**
- [ ] **Step: Verify compile**
- [ ] **Step: Commit**

---

### Task 7: ToolCache + Skill Contracts

**Files:**
- Create: `crates/claw-core-storage-tests/src/contracts/tool_cache.rs`
- Create: `crates/claw-core-storage-tests/src/contracts/skill.rs`

**tool_cache.rs:**

```rust
use i_rs_claw_core::ClawStorage;
use std::collections::HashMap;

pub async fn run(storage: &ClawStorage) -> anyhow::Result<()> {
    let agent = "tool-cache-agent";

    // Save tool cache
    let mut cache = HashMap::new();
    cache.insert("weight_tool".into(), "Weight tracking tool docs".into());
    cache.insert("todo_tool".into(), "Todo management".into());
    storage.tool_cache.save(agent, &cache).await?;

    // Load
    let loaded = storage.tool_cache.load(agent).await?;
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded.get("weight_tool").unwrap(), "Weight tracking tool docs");

    // Agent isolation
    let other = storage.tool_cache.load("other-agent").await?;
    assert!(other.is_empty() || other.len() != 2);

    // Delete
    storage.tool_cache.save(agent, &HashMap::new()).await?;
    let after = storage.tool_cache.load(agent).await?;
    assert!(after.is_empty() || after.len() == 0);

    Ok(())
}
```

**skill.rs:**

```rust
use i_rs_claw_core::ClawStorage;
use i_rs_claw_core::storage::SkillEntry;

pub async fn run(storage: &ClawStorage) -> anyhow::Result<()> {
    // Install a skill
    let entry = SkillEntry {
        name: "test-skill".into(),
        description: "A test skill".into(),
        content: "# Test Skill\n\nDo something useful.".into(),
        parameter_schema: Some(r#"{"type":"object","properties":{}}"#.into()),
    };
    storage.skills.install(&entry).await?;

    // List
    let list = storage.skills.list().await?;
    assert!(list.iter().any(|s| s.name == "test-skill"));

    // Get
    let loaded = storage.skills.get("test-skill").await?;
    assert_eq!(loaded.unwrap().description, "A test skill");

    // List executable (with parameter_schema)
    let executable = storage.skills.list_executable().await?;
    assert!(executable.iter().any(|s| s.name == "test-skill"));

    // Remove
    storage.skills.remove("test-skill").await?;
    let after = storage.skills.get("test-skill").await?;
    assert!(after.is_none());

    Ok(())
}
```

- [ ] **Step: Write tool_cache contract**
- [ ] **Step: Write skill contract**
- [ ] **Step: Verify compile**
- [ ] **Step: Commit**

---

### Task 8: File Backend Test Suite

**File:** Create `crates/claw-core-storage-tests/tests/file_contracts.rs`

```rust
use std::sync::Arc;
use tempfile::tempdir;
use i_rs_claw_core::{ClawStorage, StorageConfig, StorageBackend};
use i_rs_claw_core::storage::file::FileBackend;

/// Build a File ClawStorage backed by a temp directory.
async fn file_storage() -> Arc<ClawStorage> {
    let dir = tempdir().unwrap();
    let config = StorageConfig {
        backend: StorageBackend::File,
        file_dir: Some(dir.path().join("claw")),
        ..Default::default()
    };
    Arc::new(FileBackend::new(&config).await.unwrap())
}

#[tokio::test]
async fn session() -> anyhow::Result<()> {
    let storage = file_storage().await;
    contracts::session::run(&storage).await
}

#[tokio::test]
async fn message_log() -> anyhow::Result<()> {
    let storage = file_storage().await;
    contracts::message_log::run(&storage).await
}

#[tokio::test]
async fn memory() -> anyhow::Result<()> {
    let storage = file_storage().await;
    contracts::memory::run(&storage).await
}

#[tokio::test]
async fn stats() -> anyhow::Result<()> {
    let storage = file_storage().await;
    contracts::stats::run(&storage).await
}

#[tokio::test]
async fn tool_cache() -> anyhow::Result<()> {
    let storage = file_storage().await;
    contracts::tool_cache::run(&storage).await
}

#[tokio::test]
async fn skill() -> anyhow::Result<()> {
    let storage = file_storage().await;
    contracts::skill::run(&storage).await
}

#[tokio::test]
async fn config_store() -> anyhow::Result<()> {
    use i_rs_claw_core::ConfigStore;
    use i_rs_claw_core::storage::config_store::FileConfigStore;

    let dir = tempdir().unwrap();
    let config = StorageConfig {
        backend: StorageBackend::File,
        file_dir: Some(dir.path().join("claw")),
        ..Default::default()
    };
    let store = FileConfigStore::new(&config).await.unwrap();
    contracts::config_store::run(&store).await
}
```

- [ ] **Step: Write file_contracts.rs**
- [ ] **Step: Run the tests:** `cargo test -p claw-core-storage-tests --test file_contracts -- --test-threads=1`
- [ ] **Step: Commit**

---

### Task 9: SQLite Backend Test Suite

**File:** Create `crates/claw-core-storage-tests/tests/sqlite_contracts.rs`

```rust
use std::sync::Arc;
use tempfile::tempdir;
use i_rs_claw_core::{ClawStorage, StorageConfig, StorageBackend};

async fn sqlite_storage() -> Arc<ClawStorage> {
    let dir = tempdir().unwrap();
    let config = StorageConfig {
        backend: StorageBackend::Sqlite,
        sqlite_path: Some(dir.path().join("test.db")),
        ..Default::default()
    };
    Arc::new(ClawStorage::sqlite(&config).await.unwrap())
}

// One test per contract (same pattern as file_contracts.rs, just different factory)
// ... 6 tokio tests

#[tokio::test]
async fn session() -> anyhow::Result<()> {
    let storage = sqlite_storage().await;
    contracts::session::run(&storage).await
}
// ... etc for message_log, memory, stats, tool_cache, skill

#[tokio::test]
async fn config_store() -> anyhow::Result<()> {
    use i_rs_claw_core::ConfigStore;
    use i_rs_claw_core::storage::config_store::SqliteConfigStore;

    let dir = tempdir().unwrap();
    let config = StorageConfig {
        backend: StorageBackend::Sqlite,
        sqlite_path: Some(dir.path().join("test.db")),
        ..Default::default()
    };
    let store = SqliteConfigStore::new(&config).await.unwrap();
    contracts::config_store::run(&store).await
}
```

- [ ] **Step: Write sqlite_contracts.rs**
- [ ] **Step: Run tests:** `cargo test -p claw-core-storage-tests --test sqlite_contracts -- --test-threads=1`
- [ ] **Step: Commit**

---

## Phase 1: External Database Contracts

### Task 10: docker-compose and Test Helpers

**Files:**
- Create: `crates/claw-core-storage-tests/docker-compose.yml`
- Create: `crates/claw-core-storage-tests/src/backends/helpers.rs`

**docker-compose.yml:**

```yaml
services:
  mysql:
    image: mysql:8
    environment:
      MYSQL_ROOT_PASSWORD: test
      MYSQL_DATABASE: claw_test
    ports: ["3306:3306"]
    healthcheck:
      test: ["CMD", "mysqladmin", "ping", "-h", "localhost"]
      interval: 5s
      timeout: 3s
      retries: 10

  postgres:
    image: postgres:16
    environment:
      POSTGRES_PASSWORD: test
      POSTGRES_DB: claw_test
    ports: ["5432:5432"]
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 5s
      timeout: 3s
      retries: 10

  mongo:
    image: mongo:7
    ports: ["27017:27017"]

  redis:
    image: redis:7
    ports: ["6379:6379"]
```

**helpers.rs:**

```rust
use i_rs_claw_core::{ClawStorage, StorageConfig, StorageBackend};

/// Build MySQL storage. Requires `docker compose up mysql`.
/// Uses env vars for connection params (with defaults).
pub async fn mysql_storage() -> ClawStorage {
    let config = StorageConfig {
        backend: StorageBackend::Mysql,
        sql_url: Some(std::env::var("MYSQL_URL")
            .unwrap_or("mysql://root:test@localhost:3306/claw_test".into())),
        ..Default::default()
    };
    ClawStorage::mysql(&config).await.unwrap()
}

/// Build PostgreSQL storage.
pub async fn postgres_storage() -> ClawStorage {
    let config = StorageConfig {
        backend: StorageBackend::Postgres,
        sql_url: Some(std::env::var("PG_URL")
            .unwrap_or("postgres://postgres:test@localhost:5432/claw_test".into())),
        ..Default::default()
    };
    ClawStorage::postgres(&config).await.unwrap()
}
```

- [ ] **Step: Write docker-compose.yml**
- [ ] **Step: Write helpers.rs**
- [ ] **Step: Verify helpers compile**
- [ ] **Step: Commit**

---

### Task 11: MySQL + PostgreSQL Contract Tests

**Files:**
- Create: `crates/claw-core-storage-tests/tests/mysql_contracts.rs`
- Create: `crates/claw-core-storage-tests/tests/pg_contracts.rs`

Both follow same pattern as sqlite_contracts.rs but use the external helper factories. Gate behind `#[cfg(feature = "mysql")]` / `#[cfg(feature = "postgres")]`.

These tests are **ignored by default** — run only when docker-compose is up:

```rust
#[ignore = "requires docker compose up mysql"]
#[tokio::test]
async fn session() -> anyhow::Result<()> {
    let storage = Arc::new(backends::helpers::mysql_storage().await);
    contracts::session::run(&storage).await
}
```

- [ ] **Step: Write mysql_contracts.rs**
- [ ] **Step: Write pg_contracts.rs**
- [ ] **Step: Verify compile under features**
- [ ] **Step: Commit**

---

## Phase 2: Report System

### Task 12: Reporter Module

**Files:**
- Create: `crates/claw-core-storage-tests/src/reporter/mod.rs`
- Create: `crates/claw-core-storage-tests/src/reporter/markdown.rs`
- Create: `crates/claw-core-storage-tests/src/reporter/json.rs`

**reporter/mod.rs:**

```rust
mod markdown;
mod json;

pub struct TestRunReport {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub backends: Vec<BackendReport>,
    pub conversations: Vec<ConversationReport>,
}

pub struct BackendReport {
    pub name: String,
    pub contracts: Vec<ContractResult>,
    pub duration_ms: u64,
}

pub struct ContractResult {
    pub name: String,
    pub passed: bool,
    pub error: Option<String>,
    pub duration_ms: u64,
}

pub struct ConversationReport {
    pub tool: String,
    pub name: String,
    pub passed: bool,
    pub steps: Vec<StepResult>,
    pub total_tokens: u64,
    pub total_cost: f64,
}

pub struct StepResult {
    pub step: u32,
    pub title: String,
    pub passed: bool,
    pub tool_match: bool,
    pub storage_match: bool,
    pub error: Option<String>,
}

impl TestRunReport {
    pub fn to_markdown(&self) -> String { markdown::format(self) }
    pub fn to_json(&self) -> String { json::format(self) }
    pub fn save(&self, reports_dir: &std::path::Path) -> std::io::Result<()> {
        let ts = self.timestamp.format("%Y%m%d_%H%M%S");
        let dir = reports_dir.join(ts.to_string());
        std::fs::create_dir_all(&dir)?;
        std::fs::write(dir.join("summary.md"), self.to_markdown())?;
        std::fs::write(dir.join("summary.json"), self.to_json())?;
        // Update latest symlink
        let latest = reports_dir.join("latest");
        let _ = std::fs::remove_file(&latest);
        #[cfg(unix)]
        std::os::unix::fs::symlink(ts.to_string(), &latest)?;
        Ok(())
    }
}
```

**markdown.rs** formats the report as Markdown tables showing:

```
# 存储层测试报告 — {timestamp}

## 1. 契约测试结果

| 驱动 | 会话 | 消息日志 | 记忆 | 统计 | 配置 | 工具缓存 | 技能 |
|------|------|---------|------|------|------|---------|------|
| File | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

## 2. Backend Performance

| 操作 | File | SQLite |
```

**json.rs** outputs structured JSON.

- [ ] **Step: Write mod.rs with data structures**
- [ ] **Step: Write markdown.rs**
- [ ] **Step: Write json.rs**
- [ ] **Step: Add a `cargo test --features reporter` that generates a sample report**
- [ ] **Step: Commit**

---

## Phase 3: Conversation Script Engine

### Task 13: ScriptRunner — Loader

**Files:**
- Create: `crates/claw-core-storage-tests/src/conversations/runner/mod.rs`
- Create: `crates/claw-core-storage-tests/src/conversations/runner/loader.rs`

**script.json Schema:**

```rust
pub struct ScriptMeta {
    pub tool: String,
    pub name: String,
    pub description: String,
    pub required_capabilities: Vec<String>,
    pub storage_backends: Vec<String>,
    pub tags: Vec<String>,
}

pub struct ScriptStep {
    pub step: u32,
    pub title: String,
    pub user_message: String,
    pub expected_tool: Option<String>,
    pub expected_command: Option<String>,
    pub expected_args: Option<HashMap<String, String>>,
    pub expected_flags: Option<HashMap<String, String>>,
    pub check_reply: Option<ReplyCheck>,
    pub verify_storage: Option<StorageCheck>,
}

pub struct ReplyCheck {
    pub contains: Vec<String>,
}

pub struct StorageCheck {
    pub file_check: Option<String>,
    pub expected_state: Option<serde_json::Value>,
    pub no_new_records: Option<bool>,
}

pub struct Script {
    pub meta: ScriptMeta,
    pub steps: Vec<ScriptStep>,
}
```

Loader reads `script.json` from a scenario directory and deserializes.

- [ ] **Step: Write data structures**
- [ ] **Step: Write loader with serde deserialization**
- [ ] **Step: Commit**

---

### Task 14: ScriptRunner — Executor (Direct Mode)

**File:** Create `crates/claw-core-storage-tests/src/conversations/runner/executor.rs`

Direct mode sends messages through `AppCore` directly (no HTTP server):

```rust
pub struct DirectSession {
    core: Arc<RwLock<AppCore>>,
    session_id: String,
}

impl DirectSession {
    pub async fn new(core: Arc<RwLock<AppCore>>) -> Self { ... }

    /// Send a user message and receive the assistant reply.
    /// Returns the full message text and any tool calls made.
    pub async fn send_message(&mut self, text: &str) -> anyhow::Result<StepOutput> {
        // 1. Call core.write().await.spawn_chat(text, ...)
        // 2. Collect LlmEvent stream
        // 3. Extract final assistant message and tool calls
        // 4. Return StepOutput { reply, tool_calls, tokens }
    }
}

pub struct StepOutput {
    pub reply: String,
    pub tool_calls: Vec<ToolCallInfo>,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

pub struct ToolCallInfo {
    pub tool: String,
    pub command: Option<String>,
    pub args: HashMap<String, String>,
}
```

- [ ] **Step: Write DirectSession struct and send_message**
- [ ] **Step: Write tool call extraction logic**
- [ ] **Step: Test with a hardcoded message**
- [ ] **Step: Commit**

---

### Task 15: ScriptRunner — Verifier

**File:** Create `crates/claw-core-storage-tests/src/conversations/runner/verifier.rs`

```rust
pub struct Verifier {
    storage: Arc<ClawStorage>,
}

impl Verifier {
    pub fn new(storage: Arc<ClawStorage>) -> Self { ... }

    /// Verify tool_call matches expected values.
    pub fn check_tool_call(
        &self,
        step: &ScriptStep,
        actual: &ToolCallInfo,
    ) -> Vec<String> { // returns list of deviations
    }

    /// Verify reply text contains expected keywords.
    pub fn check_reply(
        &self,
        step: &ScriptStep,
        reply: &str,
    ) -> Vec<String> {
    }

    /// Verify storage state via ClawStorage trait.
    pub async fn check_storage(
        &self,
        step: &ScriptStep,
    ) -> Vec<String> {
    }
}
```

- [ ] **Step: Write Verifier struct**
- [ ] **Step: Implement check_tool_call, check_reply, check_storage**
- [ ] **Step: Commit**

---

### Task 16: ScriptRunner Integration

**File:** Create `crates/claw-core-storage-tests/src/conversations/runner/mod.rs`

```rust
pub struct ScriptRunner {
    script: Script,
    session: DirectSession,
    verifier: Verifier,
    results: Vec<StepResult>,
    total_tokens: u64,
    total_cost: f64,
}

impl ScriptRunner {
    pub async fn new(
        script: Script,
        core: Arc<RwLock<AppCore>>,
        storage: Arc<ClawStorage>,
    ) -> Self { ... }

    /// Run all steps, collect results.
    pub async fn run(&mut self) -> ConversationReport {
        for step in &self.script.steps {
            let output = self.session.send_message(&step.user_message).await.unwrap();
            let deviations = self.verifier.check_tool_call(step, &output);
            let storage_issues = self.verifier.check_storage(step).await;
            // ... build StepResult
        }
        // Build ConversationReport
    }
}
```

- [ ] **Step: Write ScriptRunner::run loop**
- [ ] **Step: Hardcode a mini-script and run against File backend**
- [ ] **Step: Commit**

---

## Phase 4: CLI Conversation Scripts

### Task 17: Script Template

**File:** Create `conversations/scenarios/_template/`

This serves as the canonical template for adding any new CLI tool script.

**`_template/README.md`:**

```markdown
# {工具名称} 使用引导

目标：让用户学会通过自然对话使用 {工具名称}

## 工具信息
- CLI 名称: `i-rs-xxx`
- 存储键: `"xxx"`
- 数据文件: `~/.i-rs/data/xxx.json`

## 命令列表
| 命令 | 参数 | 说明 |
|------|------|------|
| add | ... | ... |
| list | ... | ... |

## 对话步骤设计原则
1. 步骤 1: 首次记录（add）
2. 步骤 2: 查询刚才的记录（get 或 list）
3. 步骤 3: 更新或修改（update）
4. 步骤 4: 特殊命令（如有：stats/done/calendar 等）
5. 步骤 5: 删除（delete）
6. 可选：带标签/备注的高级用法
```

**`_template/script.json`** — a skeleton with comments explaining each field:

```json
{
  "meta": {
    "tool": "i-rs-xxx",
    "name": "xxx_basic_usage",
    "description": "引导用户学习使用 xxx",
    "required_capabilities": ["add", "list", "get", "update", "delete"],
    "storage_backends": ["file", "sqlite"],
    "tags": ["guided-tutorial", "xxx"]
  },
  "steps": [
    {
      "step": 1,
      "title": "记录一条数据",
      "user_message": "替换为真实的用户消息",
      "expected_tool": "i-rs-xxx",
      "expected_command": "add",
      "check_reply": {"contains": ["已记录"]},
      "verify_storage": {
        "expected_state": {"field": "value"}
      }
    }
  ]
}
```

- [ ] **Step: Create _template/ directory with README.md + script.json**
- [ ] **Step: Commit**

---

### Task 18: KV Script (01-kv)

**File:** Create `conversations/scenarios/01-kv/`

**Key characteristics:** Arbitrary key-value pairs, no dates, special commands (search, stats, copy, rename).

**Steps:**
1. `add blog_url https://example.com` → store key-value
2. `get blog_url` → retrieve by key
3. `list` → show all keys
4. `search example` → search across values
5. `delete blog_url` → remove by key
6. `add api_key sk-test --tag test --remark 测试用` → with tags

- [ ] **Step: Write README.md with natural language script**
- [ ] **Step: Write script.json with 6 steps**
- [ ] **Step: Run against File backend to verify**
- [ ] **Step: Commit**

---

### Task 19: Todo Script (02-todo)

**Key characteristics:** User-chosen name (not UUID), special `done` command (toggle), priority levels, `content` is Vec\&lt;String\&gt;.

**Steps:**
1. `add 买牛奶 --priority high` → create high-priority todo
2. `list` → show all
3. `done 买牛奶` → toggle done
4. `list --pending` → show only pending
5. `add 写周报 --content 整理本周数据` → with content
6. `delete 买牛奶` → remove

- [ ] **Step: Write README.md + script.json (6 steps)**
- [ ] **Step: Run against File backend**
- [ ] **Step: Commit**

---

### Task 20: Weight Script (03-weight)

**Key characteristics:** Numeric data, `--date` flag, `list --stats` for min/max/avg, `list --chart` for ASCII chart.

**Steps:**
1. `add 75.5` → record today's weight
2. `list --days 7` → show recent entries
3. `list --stats` → show min/max/avg
4. `add 74.8 --tag morning` → with morning tag

- [ ] **Step: Write README.md + script.json**
- [ ] **Step: Run against File backend**
- [ ] **Step: Commit**

---

### Task 21: Water Script (04-water)

**Key characteristics:** `amount_ml` (i32), no custom date, `drank_at` auto-set, today's total tracking.

**Steps:**
1. `add 300` → record 300ml
2. `add 500` → record another 500ml
3. `list` → show all entries
4. Ask "我今天喝了多少？" → agent should compute total

- [ ] **Step: Write README.md + script.json**
- [ ] **Step: Run against File backend**
- [ ] **Step: Commit**

---

### Task 22: Sleep Script (05-sleep)

**Key characteristics:** bedtime + wake_time (HH:MM), quality (1-5 with labels), computed duration, stats.

**Steps:**
1. `add 23:00 07:00 4` → sleep quality "Good"
2. `list` → show entries with duration
3. `stats` → show avg_duration, avg_quality
4. `add 00:30 08:00 3 --tag weekend` → with tag

- [ ] **Step: Write README.md + script.json**
- [ ] **Step: Run against File backend**
- [ ] **Step: Commit**

---

### Task 23: Meal Script (06-meal)

**Key characteristics:** `meal_type` (free string), `--food` (required), optional `--calories`, no `updated_at` field.

**Steps:**
1. `add breakfast --food 面包牛奶 --calories 350` → record meal
2. `list --date today` → show today's meals
3. `add lunch --food 米饭炒菜 --calories 650 --tag 外卖` → with tag
4. Ask "我今天吃了多少卡路里？" → agent should sum

- [ ] **Step: Write README.md + script.json**
- [ ] **Step: Run against File backend**
- [ ] **Step: Commit**

---

### Task 24: Mood Script (07-mood)

**Key characteristics:** 7-level enum (terrible→amazing), `--date` flag, `list --calendar`, auto-stats.

**Steps:**
1. `add good` → record good mood
2. `add great --date yesterday` → backdate
3. `list --days 7` → show with stats
4. `list --calendar` → show calendar view

- [ ] **Step: Write README.md + script.json**
- [ ] **Step: Run against File backend**
- [ ] **Step: Commit**

---

### Task 25: Sit Script (08-sit)

**Key characteristics:** `duration_minutes`, auto-calculates `started_at` and `ended_at`.

**Steps:**
1. `add 45` → record 45 min sitting session
2. `list` → show with start/end times
3. `add 60 --tag work` → with tag

- [ ] **Step: Write README.md + script.json**
- [ ] **Step: Run against File backend**
- [ ] **Step: Commit**

---

### Task 26: Pig Script (09-pig)

**Key characteristics:** `food_name`, optional `--description`, no custom date.

**Steps:**
1. `add 火锅 --description 想和同事一起去` → record craving
2. `list` → show all cravings
3. `add 奶茶 --tag 下午茶` → with tag

- [ ] **Step: Write README.md + script.json**
- [ ] **Step: Run against File backend**
- [ ] **Step: Commit**

---

### Task 27: Spark Script (10-spark)

**Key characteristics:** `content`, optional `--source`, simple CRUD, no special commands.

**Steps:**
1. `add 人生就像骑自行车 --source 爱因斯坦` → capture inspiration
2. `list` → show all inspirations
3. `add 保持简单 --source 达芬奇 --tag 哲学` → with tag and source

- [ ] **Step: Write README.md + script.json**
- [ ] **Step: Run against File backend**
- [ ] **Step: Commit**

---

## Phase 5: Advanced Reports & Remaining Backends

### Task 28: Agent Intent Analysis Report

**File:** Modify `reporter/markdown.rs`

Add "Agent 意图识别分析" section comparing expected vs actual tool calls per step.

For each deviation, output:
```
工具: {tool}
步骤 {N}: "{user_message}"
  → 期望: {expected_tool} {expected_command} {expected_args}
  → 实际: {actual_tool} {actual_command} {actual_args}
  → 偏差: {description}
  → 改进建议: {suggestion}
```

- [ ] **Step: Implement deviation analysis**
- [ ] **Step: Integrate into ScriptRunner results**
- [ ] **Step: Commit**

---

### Task 29: System Prompt Evolution Tracking

**File:** Modify `conversations/runner/executor.rs`

Capture the system prompt before and after each step. Report how `{{TOOL_INDEX}}`, `{{USER_MEMORY}}`, `{{USER_PROFILE}}`, `{{SKILLS}}`, `{{HOT_TOOLS}}` change across steps.

```rust
pub struct SystemPromptSnapshot {
    pub step: u32,
    pub raw_length: usize,
    pub sections: HashMap<String, String>, // "TOOL_INDEX", "USER_MEMORY", etc.
}
```

Report format:
```
消息 1: TOOL_INDEX=[8 tools, 2,100 chars], USER_MEMORY=空, SKILLS=空 → 2,430 tokens
消息 2: TOOL_INDEX=[8 tools, 2,100 chars], USER_MEMORY=有(体重记录, 45 chars) → 2,580 tokens
```

- [ ] **Step: Add system prompt capture to DirectSession**
- [ ] **Step: Add evolution section to report**
- [ ] **Step: Commit**

---

### Task 30: Mongo/Redis Contract Tests

**Files:**
- Create: `crates/claw-core-storage-tests/src/backends/mongo.rs`
- Create: `crates/claw-core-storage-tests/src/backends/redis.rs`
- Create: `tests/mongo_contracts.rs` (ignored by default)
- Create: `tests/redis_contracts.rs` (ignored by default)

Requires `#[cfg(feature = "mongo")]` and `#[cfg(feature = "redis")]` gates.

- [ ] **Step: Implement Mongo test helper + contract tests**
- [ ] **Step: Implement Redis test helper + contract tests**
- [ ] **Step: Verify compile under features**
- [ ] **Step: Commit**

---

### Task 31: Cross-Tool Scenarios

**Files:**
- Create: `conversations/scenarios/cross-tool/daily-checkin/`
- Create: `conversations/scenarios/cross-tool/health-summary/`

**daily-checkin** — "早间打卡" scenario:
1. Record mood: "今天心情不错" → `mood add good`
2. Record weight: "早上空腹体重 74.5" → `weight add 74.5 --tag morning`
3. Record water: "喝了一杯水 300ml" → `water add 300`

**health-summary** — "健康周报":
1. Query weight stats: "这周体重变化怎么样？"
2. Query sleep stats: "睡眠质量呢？"
3. Query meal: "我这两天吃了什么？"
4. Query sit: "久坐情况如何？"

- [ ] **Step: Write daily-checkin script**
- [ ] **Step: Write health-summary script**
- [ ] **Step: Run against File backend**
- [ ] **Step: Commit**

---

### Task 32: Performance Comparison Report

**Files:** Modify `reporter/markdown.rs`

Add a performance section to the report comparing operation latency across backends:

```markdown
## 性能对比

| 操作 | File | SQLite | MySQL | PG |
|------|------|--------|-------|----|
| append_batch(100 msgs) | 2.1ms | 1.8ms | n/a | n/a |
```

Requires wrapping each contract operation with `std::time::Instant` timing.

- [ ] **Step: Add timing to contract runner**
- [ ] **Step: Format performance table in report**
- [ ] **Step: Commit**

---

## Summary

| Phase | Tasks | Effort | Dependencies |
|-------|-------|--------|-------------|
| P0: Foundation | 1-9 (contract tests) | 3 days | None |
| P1: External DB | 10-11 | 1 day | docker-compose |
| P2: Report System | 12 | 1 day | P0 |
| P3: Script Engine | 13-16 | 2 days | P0 |
| P4: CLI Scripts | 17-27 | 3 days | P0, P3 |
| P5: Advanced | 28-32 | 2 days | P2, P3, P4 |

Total: ~32 tasks across 6 phases.
