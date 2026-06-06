# claw 存储层关键 Bug 修复 — 执行计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复 STORAGE_REVIEW.md 中 Phase 1 的 6 项关键问题 (4 🔴 数据丢失 + 1 🟠 性能 + 1 trait 重构)

**架构:** 6 项修复按文件组织，无交叉依赖，可顺序或并行执行。Trait 重构 (#7+#8) 最后做因为它影响所有后端。

**Tech Stack:** Rust, file.rs SQL mod.rs + 宏, session.rs, storage/mod.rs + 4 后端

---

### Task 1: #4 — index.json 损坏保护 (file.rs)

**Files:**
- Modify: `crates/claw/src/storage/file.rs:126-140`
- Modify: `crates/claw/src/storage/file.rs:145-155`

- [ ] **Step 1: 修改 `load_all` 使其在损坏时返回 `Err` 而非静默空列表**

将原 `load_all` 方法：

```rust
let content = std::fs::read_to_string(&path)?;
serde_json::from_str(&content)
    .inspect_err(|e| tracing::error!("index.json 损坏，无法解析 ({}); 返回空列表以允许重建", e))
    .or_else(|_| Ok(Vec::new()))
```

改为：

```rust
let content = std::fs::read_to_string(&path)?;
serde_json::from_str(&content)
    .inspect_err(|e| tracing::error!("index.json 损坏: {} — 不会静默清空", e))
    .map_err(|e| anyhow::anyhow!("index.json 损坏: {}", e))
```

- [ ] **Step 2: 在 `save_all` 中添加备份机制**

在 `save_all` 中，序列化成功后、原子写入前添加备份：

```rust
async fn save_all(&self, sessions: &[crate::session::SessionMeta]) -> anyhow::Result<()> {
    let path = index_path(&self.claw_dir);
    let content = serde_json::to_string_pretty(sessions)?;
    blocking(move || {
        ensure_dir(&path)?;
        // 备份旧文件 (如果存在)
        if path.exists() {
            let bak = path.with_extension("json.bak");
            std::fs::copy(&path, &bak).ok(); // 非致命错误
        }
        atomic_write(&path, &content).map_err(anyhow::Error::from)
    })
    .await
}
```

- [ ] **Step 3: 检查 `cargo check` 通过**

Run: `cargo check -p i-rs-claw`
Expected: 0 errors

---

### Task 2: #5 — StatsRepo::prune 加锁 (file.rs)

**Files:**
- Modify: `crates/claw/src/storage/file.rs:344-389`

- [ ] **Step 1: 在 `prune` 的 `blocking` 闭包开头添加 STATS_LOCK 保护**

原 `prune` 开头：

```rust
async fn prune(&self, keep_days: u32) -> anyhow::Result<usize> {
    let path = stats_path(&self.claw_dir);
    blocking(move || {
        if keep_days == 0 || !path.exists() {
```

改为：

```rust
async fn prune(&self, keep_days: u32) -> anyhow::Result<usize> {
    let path = stats_path(&self.claw_dir);
    blocking(move || {
        let _guard = lock_guard(&STATS_LOCK);
        if keep_days == 0 || !path.exists() {
```

- [ ] **Step 2: 确认 `cargo check` 通过**

Run: `cargo check -p i-rs-claw`
Expected: 0 errors

---

### Task 3: #6 — append_new_messages 更新 message_count (session.rs)

**Files:**
- Modify: `crates/claw/src/session.rs:367-385`

- [ ] **Step 1: 在 `append_new_messages` 成功分支更新 `message_count`**

原代码在 `Ok(())` 分支只更新 cursor：

```rust
Ok(()) => {
    self.saved_cursors.insert(session_id.to_string(), messages.len());
}
```

改为：

```rust
Ok(()) => {
    self.saved_cursors.insert(session_id.to_string(), messages.len());
    // 更新 SessionMeta.message_count
    if let Some(idx) = self.index.get(session_id) {
        if let Some(meta) = self.sessions.get_mut(*idx) {
            meta.message_count += new_msgs.len();
            meta.updated_at = chrono::Utc::now().timestamp();
        }
    }
    // 持久化 message_count
    self.save_index();
}
```

- [ ] **Step 2: 确认 `cargo check` 通过**

Run: `cargo check -p i-rs-claw`
Expected: 0 errors

---

### Task 4: #3 — SQL load 查询逻辑错误 (sql/mod.rs)

**Files:**
- Modify: `crates/claw/src/storage/sql/mod.rs:148-158`

- [ ] **Step 1: 将 `load` 改为子查询 + LIMIT**

原 SQL:

```sql
SELECT payload FROM message_log WHERE session_id = ? \
 AND seq > (SELECT COALESCE(MAX(seq), 0) FROM message_log WHERE session_id = ?) - ? \
 ORDER BY seq ASC
```

改为:

```sql
SELECT payload FROM (
    SELECT payload FROM message_log
    WHERE session_id = ?
    ORDER BY seq DESC
    LIMIT ?
) sub
ORDER BY seq ASC
```

Rust 代码修改：

```rust
async fn load(
    &self,
    session_id: &str,
    limit: usize,
) -> anyhow::Result<Vec<crate::app::Message>> {
    let limit_i64 = if limit >= i64::MAX as usize { i64::MAX } else { limit as i64 };
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT payload FROM ( \
         SELECT payload FROM message_log \
         WHERE session_id = ? \
         ORDER BY seq DESC \
         LIMIT ? \
         ) sub ORDER BY seq ASC",
    )
    .bind(session_id)
    .bind(limit_i64)
    .fetch_all(&self.db.pool)
    .await?;
    Ok(rows
        .into_iter()
        .filter_map(|(p,)| {
            let value: serde_json::Value = serde_json::from_str(&p).ok()?;
            let rec = crate::message::StoredRecord {
                seq: 0,
                ts: 0,
                schema_v: 1,
                payload: value,
            };
            rec.to_message()
        })
        .collect())
}
```

- [ ] **Step 2: 确认 `cargo check` 通过**

Run: `cargo check -p i-rs-claw`
Expected: 0 errors

---

### Task 5: #7+#8 — SessionRepo trait 重构

**Files:**
- Modify: `crates/claw/storage/mod.rs` — trait 定义
- Modify: `crates/claw/storage/file.rs` — FileSessionStore 实现
- Modify: `crates/claw/storage/sql/mod.rs` — SQL SessionRepo 宏实现
- Modify: `crates/claw/storage/mongo.rs` — MongoSessionStore 实现
- Modify: `crates/claw/storage/redis.rs` — RedisSessionStore 实现

- [ ] **Step 1: 改造 trait 定义 — 添加必需方法，移除默认实现**

在 `storage/mod.rs` 中：

```rust
#[async_trait]
#[allow(dead_code)]
pub trait SessionRepo: Send + Sync {
    /// Load all session metadata.
    async fn load_all(&self) -> anyhow::Result<Vec<crate::session::SessionMeta>>;
    /// Atomically replace all session metadata.
    async fn save_all(&self, sessions: &[crate::session::SessionMeta]) -> anyhow::Result<()>;

    /// Get a single session by ID.
    async fn get_one(&self, id: &str) -> anyhow::Result<Option<crate::session::SessionMeta>>;
    /// Upsert a single session metadata.
    async fn upsert(&self, session: &crate::session::SessionMeta) -> anyhow::Result<()>;
    /// Delete a single session by ID.
    async fn delete_one(&self, id: &str) -> anyhow::Result<()>;
    /// Count sessions.
    async fn count(&self) -> anyhow::Result<usize>;
}
```

- [ ] **Step 2: 检查所有消费者代码是否使用了旧的 `get`/`delete`/`count` 方法**

Run: `grep -rn '\.get(' crates/claw/src/ | grep -i session`
Run: `grep -rn '\.delete(' crates/claw/src/ | grep -i session`
Run: `grep -rn '\.count(' crates/claw/src/ | grep -i session`

如果消费者代码使用了 `repo.get(id)` 而非 `repo.get_one(id)`，将它们改为新命名。

- [ ] **Step 3: FileSessionStore 实现新方法**

在 `file.rs` 中 `impl SessionRepo for FileSessionStore` 添加:

```rust
async fn get_one(&self, id: &str) -> anyhow::Result<Option<crate::session::SessionMeta>> {
    let all = self.load_all().await?;
    Ok(all.into_iter().find(|s| s.id == id))
}

async fn upsert(&self, session: &crate::session::SessionMeta) -> anyhow::Result<()> {
    let mut all = self.load_all().await?;
    if let Some(pos) = all.iter().position(|s| s.id == session.id) {
        all[pos] = session.clone();
    } else {
        all.push(session.clone());
    }
    self.save_all(&all).await
}

async fn delete_one(&self, id: &str) -> anyhow::Result<()> {
    let mut all = self.load_all().await?;
    all.retain(|s| s.id != id);
    self.save_all(&all).await
}

async fn count(&self) -> anyhow::Result<usize> {
    Ok(self.load_all().await?.len())
}
```

- [ ] **Step 4: SQL SessionRepo 实现新方法 (宏中)**

在 `sql/mod.rs` 的 `define_sql_stores!` 宏的 `SessionRepo` impl 中添加:

```rust
async fn get_one(&self, id: &str) -> anyhow::Result<Option<crate::session::SessionMeta>> {
    let row: Option<SessionRow> = sqlx::query_as(
        "SELECT id, title, agent_id, state, created_at, updated_at, message_count \
         FROM sessions WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&self.db.pool)
    .await?;
    Ok(row.map(|r| r.into()))
}

async fn upsert(&self, session: &crate::session::SessionMeta) -> anyhow::Result<()> {
    sqlx::query($upsert_session)
        .bind(&session.id).bind(&session.title).bind(&session.agent_id)
        .bind(serde_json::to_string(&session.state).unwrap_or_default())
        .bind(session.created_at).bind(session.updated_at)
        .bind(session.message_count as i64)
        .execute(&self.db.pool).await?;
    Ok(())
}

async fn delete_one(&self, id: &str) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind(id)
        .execute(&self.db.pool).await?;
    Ok(())
}

async fn count(&self) -> anyhow::Result<usize> {
    let (count,): (i64,) = sqlx::query_scalar("SELECT COUNT(*) FROM sessions")
        .fetch_one(&self.db.pool).await?;
    Ok(count as usize)
}
```

- [ ] **Step 5: MongoSessionStore 实现新方法**

在 `mongo.rs` 的 `impl SessionRepo for MongoSessionStore` 中添加:

```rust
async fn get_one(&self, id: &str) -> anyhow::Result<Option<crate::session::SessionMeta>> {
    let doc = self.db.db.collection::<Document>("sessions")
        .find_one(doc! { "_id": id })
        .await?;
    doc.as_ref().map(doc_to_session_meta).transpose()
}

async fn upsert(&self, session: &crate::session::SessionMeta) -> anyhow::Result<()> {
    let mut doc = session_meta_to_doc(session);
    doc.insert("_id", session.id.clone());
    self.db.db.collection::<Document>("sessions")
        .update_one(
            doc! { "_id": &session.id },
            doc! { "$set": doc },
        )
        .upsert(true)
        .await?;
    Ok(())
}

async fn delete_one(&self, id: &str) -> anyhow::Result<()> {
    self.db.db.collection::<Document>("sessions")
        .delete_one(doc! { "_id": id })
        .await?;
    Ok(())
}

async fn count(&self) -> anyhow::Result<usize> {
    let count = self.db.db.collection::<Document>("sessions")
        .estimated_document_count()
        .await?;
    Ok(count as usize)
}
```

- [ ] **Step 6: RedisSessionStore 实现新方法**

在 `redis.rs` 的 `impl SessionRepo for RedisSessionStore` 中添加:

```rust
async fn get_one(&self, id: &str) -> anyhow::Result<Option<crate::session::SessionMeta>> {
    let mut conn = self.backend.conn.clone();
    let json: Option<String> =
        redis::cmd("HGET").arg(session_key(id)).arg("__json").query_async(&mut conn).await?;
    Ok(json.and_then(|j| serde_json::from_str(&j).ok()))
}

async fn upsert(&self, session: &crate::session::SessionMeta) -> anyhow::Result<()> {
    let mut conn = self.backend.conn.clone();
    let json = serde_json::to_string(session)?;
    redis::pipe()
        .atomic()
        .cmd("HSET").arg(session_key(&session.id)).arg("__json").arg(&json).ignore()
        .cmd("ZADD").arg(SESSIONS_ZSET).arg(session.updated_at).arg(&session.id).ignore()
        .query_async::<()>(&mut conn).await?;
    Ok(())
}

async fn delete_one(&self, id: &str) -> anyhow::Result<()> {
    let mut conn = self.backend.conn.clone();
    redis::pipe()
        .atomic()
        .cmd("DEL").arg(session_key(id)).ignore()
        .cmd("ZREM").arg(SESSIONS_ZSET).arg(id).ignore()
        .query_async::<()>(&mut conn).await?;
    Ok(())
}

async fn count(&self) -> anyhow::Result<usize> {
    let mut conn = self.backend.conn.clone();
    let count: i64 = redis::cmd("ZCARD").arg(SESSIONS_ZSET).query_async(&mut conn).await?;
    Ok(count as usize)
}
```

- [ ] **Step 7: 确认 `cargo check` 通过**

Run: `cargo check --features "sqlite,mysql,postgres,mongo,redis,dashboard" -p i-rs-claw`
Expected: 0 errors

---

### Task 6: 验证

**Files:**
- No file changes

- [ ] **Step 1: cargo check**

Run: `cargo check -p i-rs-claw`
Expected: 0 errors, 0 warnings

- [ ] **Step 2: cargo clippy**

Run: `cargo clippy -p i-rs-claw -- -D warnings`
Expected: clean

- [ ] **Step 3: cargo test**

Run: `cargo test -p i-rs-claw -- --test-threads=1`
Expected: 206 tests pass

- [ ] **Step 4: Commit**

```bash
git add crates/claw/src/storage/mod.rs crates/claw/src/storage/file.rs crates/claw/src/storage/sql/mod.rs crates/claw/src/storage/mongo.rs crates/claw/src/storage/redis.rs crates/claw/src/session.rs
git commit -m "fix(claw/storage): resolve 6 critical bugs from STORAGE_REVIEW

Phase 1 fixes covering data-loss, concurrency, SQL correctness,
and SessionRepo trait improvements.

Constraint: STORAGE_REVIEW.md audit (2026-06-06)
Constraint: Phase 1 — critical correctness only
Rejected: Full trait rewrite with generics | too large for Phase 1
Rejected: Async SessionManager refactor | Phase 3 scope

Fix #4: index.json corruption no longer silently clears sessions
- load_all returns Err on parse failure instead of empty Vec
- save_all backs up old index.json to index.json.bak before overwrite

Fix #5: StatsRepo::prune now holds STATS_LOCK
- Prevents concurrent upsert_batch data loss during prune

Fix #6: append_new_messages updates SessionMeta.message_count
- Also updates updated_at and persists via save_index()

Fix #3: SQL load uses DESC LIMIT + subquery
- Eliminates MAX(seq)-limit integer underflow
- Fixes logical error when limit < MAX(seq) or seq gaps exist

Fix #7+#8: SessionRepo trait now has required get_one/upsert/delete_one/count
- Adds efficient single-row operations for SQL, Mongo, Redis
- File backend falls back to load_all/save_all internally
- Removes default impls that forced full-index scan

Confidence: high
Scope-risk: moderate
Directive: SessionRepo consumers using get()/delete() must migrate
  to get_one()/delete_one(). The old names are removed from the trait.
Tested: cargo check (0 errors), cargo clippy (0 warnings)
Not-tested: Mongo/Redis backends (no CI integration for those backends)"
```
