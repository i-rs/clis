# claw Storage 抽象层审查报告 (v3)

**审查范围:** `crates/claw/src/storage/` 全 5 后端 (file / sql / mongo / redis)、`message/`、`session.rs` + gateway 消费者。

**审查日期:** 2026-06-06 (v3, 基于 commits `547b8576` ~ `859858f4`)

**v3 变更:** 基于 Phase 1.5 + Phase 2 + Phase 3 (#16) 修复后的全量重审。所有 5 后端 compile 通过 + 全部 tests pass。发现 25 个新问题/遗留问题。

---

## 1. 执行摘要

**整体评价:** 抽象层架构坚实 — 8 个 trait 设计合理，5 个后端全部通过 trait 一致性检查，383 个测试通过，clippy 0 警告。

**剩余风险:** 存在 3 个 CRITICAL 数据丢失/一致性 bug 和 6 个 HIGH 级问题。最严重的涉及:
- 序列号并发竞争（所有 SQL 后端）
- 静默数据损坏（File 后端 deserialize 失败）
- PostgreSQL 运行时崩溃（JSONB 类型不匹配）
- SessionManager 写穿导致数据丢失

---

## 2. 已修复问题总览

| # | 问题 | 修复版本 | 验证 |
|---|------|----------|------|
| #1 | PostgreSQL `?` 占位符 → `$N` | Phase 2 (#1) | ✅ Postgres 编译通过 |
| #2 | SQL `MAX(seq)+1` 并发竞争 → `FOR UPDATE` | Phase 2 (#2) | ✅ 已加 `$select_max_seq` + 方言特定锁 |
| #3 | SQL load 查询逻辑错误 + seq 列缺失 | Phase 1 | ✅ `SELECT payload, seq` + 子查询修复 |
| #4 | index.json 损坏时静默清空 | Phase 1.5 | ✅ 存储层 `.bak` + `Result` 返回 |
| #5 | `StatsRepo::prune` 缺锁 | Phase 1 | ✅ `STATS_LOCK` 已加 |
| #6 | `append_new_messages` 不更新 `message_count` | Phase 1 | ✅ 已加更新逻辑 |
| #7 | SessionRepo 缺单行操作方法 | Phase 1 | ✅ 已加 `get_one`/`upsert`/`delete_one`/`count` |
| #8 | SessionRepo 默认实现不安全 | Phase 1 | ✅ 已移除默认实现，改为必需方法 |
| #16 | search 逻辑 4 后端重复 ~48 行 × 4 | Phase 3 | ✅ 提取 `scan_records_for_query()` |
| N1-N5 | Phase 1 引入的回归问题 | Phase 1.5 | ✅ 全部修复 |
| MySQL 编译错误 | `or_else` 类型不匹配 | Chore | ✅ 修复 |

---

## 3. 新发现 / 遗留问题 (按严重性排序)

### 3.1 🔴 CRITICAL: SessionManager `unwrap_or_default()` 在 `load_all` 失败时写穿数据

**位置:** `session.rs:122`

```rust
let sessions = crate::utils::sync_block_on(async { storage.sessions.load_all().await })
    .inspect_err(|e| {
        tracing::error!("加载会话列表失败: {} — 以空列表启动，不会覆盖损坏文件", e);
    })
    .unwrap_or_default();  // ← 返回空 Vec，但 SessionManager 仍可写！
```

**问题:** 日志说 "不会覆盖损坏文件"，但实际行为是: 空 session 列表加载后，下一次 `save_session` 或 `save_index` 会正常写入 `index.json`，**覆盖损坏文件为新文件（含空内容）**，丢弃所有原有数据。`.bak` 备份存在但无自动恢复逻辑。

**修复:** `load_all` 失败时设置内部 `readonly` 标志，禁止所有写操作直到用户明确执行 recovery 命令。或者 `with_storage` 返回 `Err` 让调用方完全拒绝启动。

---

### 3.2 🔴 CRITICAL: SQL `message_log` 无 `UNIQUE(session_id, seq)` 约束 — 首次写入并发竞争

**位置:** `sql/sqlite.rs:83-88`, `sql/mysql.rs:150`, `sql/postgres.rs:139-144`

**问题:** Phase 2 加的 `FOR UPDATE` 仅在目标行已存在时有效。当 session 首次写入 (`MAX(seq) = NULL`) 时:
1. `SELECT COALESCE(MAX(seq), 0)` → 返回 0
2. PG `FOR UPDATE` 锁定 0 行（无幻读保护在 READ COMMITTED）
3. SQLite `BEGIN` 默认为 deferred（WAL 模式下不阻止并发写）
4. 两个事务同时读到 `next_seq=0`，都写入 `seq=1` → **重复 seq**

**修复:** 所有方言添加 `UNIQUE(session_id, seq)` 约束 + 写入侧添加 `ON CONFLICT` 重试。

---

### 3.3 🔴 CRITICAL: PostgreSQL `payload JSONB` 被读取为 `String` — 运行时崩溃

**位置:** `sql/mod.rs:185-196` (load), `sql/mod.rs:247-250` (search), `sql/postgres.rs:134` (schema)

**问题:** PostgreSQL schema 定义 `payload JSONB NOT NULL`。但 SQL load 和 search 查询用:
```rust
let rows: Vec<(String,)> = sqlx::query_as("SELECT payload FROM ...")
```
sqlx binary 协议对 JSONB 列返回 `serde_json::Value`，不是 `String`。**PG 后端在首次读取 message_log 时必定崩溃。**

另外，`search` 中的 `WHERE LOWER(payload) LIKE ?` 在 JSONB 列上行为与 TEXT 不同（JSONB 键排序、空格规范化 → 搜索文本可能与原始 JSON 不一致）。

**修复:** PostgreSQL schema 的 `payload` 改用 `TEXT NOT NULL`（与 SQLite/MySQL 一致），或所有读取端改用 `serde_json::Value` 类型。

---

### 3.4 🔴 CRITICAL: `save_session` 在 session 不在 index 时静默 no-op

**位置:** `session.rs:448-450`

```rust
fn save_session(&self, id: &str) {
    let Some(idx) = self.find_index(id) else {
        return; // ← 静默返回，不保存，不报错
    };
```

**问题:** 如果 session 从 `sessions` Vec 中被删除（通过 `delete_session`）但在调用 `save_session` 之前 `find_index` 找不到它，该 session 的保存静默失败。这可能导致 `updated_at` / `message_count` / `state` 变更丢失。

**修复:** 至少用 `tracing::warn!` 记录这种异常；理想情况是返回 `Result`。

---

### 3.5 🟠 HIGH: File 后端 `unwrap_or_default()` 在 deserialize 失败时静默返回空数据

**位置:** `file.rs:218` (`ApiCacheRepo::load`), `file.rs:266` (`PlanStepsRepo::load`), `file.rs:606` (`ToolCacheRepo::load`)

```rust
Ok(serde_json::from_str(&content).unwrap_or_default())
```

**问题:** JSON 解析失败时返回 `Ok(vec![])` / `Ok(HashMap::new())` — 调用方认为数据为空，下一次 `save` 会覆盖损坏文件。与 `SessionRepo::load_all` (Phase 1.5 后返回 `Err`) 不一致。

**修复:** `map_err(|e| anyhow!("..."))?` 传播错误。

---

### 3.6 🟠 HIGH: File 后端 TOCTOU 竞态在 `SessionRepo::upsert` / `delete_one`

**位置:** `file.rs:160-178`

**问题:** 两个方法都走 `load_all()` → 修改内存 → `save_all()`。没有锁保护。两个并发调用方读到相同快照，各自修改，后写覆盖先写 —— 丢失中间变更。

**修复:** 对 session 变更操作加 `SESSION_LOCK`（类似 `STATS_LOCK`）。

---

### 3.7 🟠 HIGH: File 后端 I/O 错误被 `map_while(Result::ok)` 静默截断

**位置:** `file.rs:689` (`load`), `file.rs:740` (`search`), `file.rs:776` (`count`)

```rust
BufReader::new(file).lines().map_while(Result::ok)
```

**问题:** 读取到一半遇到 I/O 错误时（坏块、网络 FS 断开），`map_while(Result::ok)` 停止迭代但不报错。调用方得到"成功"的**部分结果**。

**修复:** 收集 lines → 检查是否有任何错误 → 有错则返回 `Err`。

---

### 3.8 🟠 HIGH: File 后端 search 与 load_all 对 `index.json` 损坏的处理不一致

**位置:** `file.rs:718-724`

```rust
std::fs::read_to_string(&index_path)
    .ok()
    .and_then(|c| serde_json::from_str(&c).ok())
    .unwrap_or_default()
```

**问题:** `SessionRepo::load_all` 在 Phase 1.5 后正确返回 `Err`。但 `search` 仍吞掉所有错误，返回 0 结果，且无任何日志。

**修复:** 统一使用 `load_all` trait 方法，或至少 `tracing::error!` + 空结果。

---

### 3.9 🟠 HIGH: Gateway 绕过 `append_new_messages` 直接调 `message_log().append_batch()`

**位置:** `gateway/mod.rs:283-296`, `session.rs:363`

```rust
let log = core.session_mgr.message_log();
log.append_batch(&session_uuid, &msgs).await  // ← 绕过 SessionManager
```

**问题:**
1. `saved_cursors` 不更新 → TUI 打开该 session 时 `append_new_messages` 会把已有消息重新写入一次（重复）
2. `SessionMeta.message_count` 不更新 → session 列表中计数错误
3. `SessionMeta.updated_at` 不更新 → 最后活跃时间错误

**修复:** Gateway 应该通过 `SessionManager` 的方法持久化消息（即使需要新增方法），而不是绕过去直接调 MessageLog。

---

### 3.10 🟠 HIGH: SQL search N+1 查询模式

**位置:** `sql/mod.rs:233-259`

**问题:** 对每个候选 session 执行 2 个额外查询（sessions 表 + message_log 表）。100 个候选 = 201 次查询。message_log 查询加载**全部消息**到内存（10,000 条/session → 1M 条加载然后 Rust 扫描）。

**修复:** JOIN sessions 表一次查询完成；message_log 只加载匹配行的上下文而非全部。

---

### 3.11 🟡 MEDIUM: `.bak` 文件死代码 — 从未恢复也从未清理

**位置:** `file.rs:146-149`

```rust
let bak = path.with_extension("json.bak");
std::fs::copy(&path, &bak).ok();  // ← 总是忽略错误
```

**问题:**
- `atomic_write` 本身就 crash-safe（write-tmp-rename），`.bak` 不提供额外保护
- `.bak` 从未在恢复时被读取/使用
- `.bak` 从未被清理，在 `~/.i-rs/data/` 中无限累积

**修复:** 删除 `.bak` 创建逻辑，或在恢复时读取 `.bak` 文件。

---

### 3.12 🟡 MEDIUM: SQL `save_all` / `upsert_batch` 单行操作（非批量）

**位置:** `sql/mod.rs:76-83` (`save_all` 删除), `sql/mod.rs:358-369` (`upsert_batch`)

**问题:** `save_all` 对每个过期 session 发一条 DELETE；`upsert_batch` 对每条 token 记录发一条 UPSERT。N 个记录 = N 次网络往返。

**修复:** `save_all` 用 `DELETE FROM sessions WHERE id IN (...)`；`upsert_batch` 用事务或批量插入。

---

### 3.13 🟡 MEDIUM: MongoDB `append_batch` 非原子（seq counter + insert 分两步）

**位置:** `mongo.rs:233-277`

**问题:** `findOneAndUpdate` (seq 递增) 和 `insert_many` (消息写入) 是两个独立操作。如果 `insert_many` 失败，seq 计数器已递增，但消息未写入 → seq 空洞。违反 trait 契约: "all-or-nothing"。

**修复:** 用 MongoDB transaction (`session.with_transaction()`)。

---

### 3.14 🟡 MEDIUM: mongo/redis `save_all(empty)` 留下孤儿数据

**位置:** `mongo.rs:122-131`, `redis.rs:120-154`

**问题:** 传入空 sessions 时只清空 sessions 表，不清空 `message_log` / `seq_counters`。如果因数据损坏而重建 sessions 为空，孤儿消息数据永远留在 DB 中。

**修复:** 空 sessions 时也清空 `message_log` + `seq_counters`。

---

### 3.15 🟡 MEDIUM: `serde_json::to_string(&s.state).unwrap_or_default()` 静默损坏状态

**位置:** `sql/mod.rs:88,110`, `mongo.rs:202`

**问题:** 如果 `SessionState` 序列化失败（如添加了不可序列化的变体），状态字段保存为空字符串/空文档。重新加载时恢复为 `Default::default()` → **状态静默变更**。

**修复:** 使用 `?` 传播序列化错误。

---

### 3.16 🟡 MEDIUM: `append_batch` 不更新 session 的 `message_count` 列

**位置:** `sql/mod.rs:140-171` vs `session.rs:384-392`

**问题:** `append_batch` 插入消息但 transaction 内不更新对应 session 行的 `message_count`。SessionManager 在 Rust 侧更新后调 `save_session`，但若进程在此之间崩溃，DB 中 `message_count` 仍是旧值。

**修复:** `append_batch` 内 `UPDATE sessions SET message_count = message_count + N WHERE id = ?`。

---

### 3.17 🟢 LOW: 次要问题汇总

| # | 严重性 | 位置 | 描述 |
|---|--------|------|------|
| L1 | LOW | `file.rs:146-149` | `std::fs::copy(...).ok()` 忽略备份创建错误 |
| L2 | LOW | `file.rs:379` | `keep_days as i64 * 86400` 可能溢出（需要 >68 年） |
| L3 | LOW | `file.rs:101-105` | `extract_timestamp` 每行解析完整 JSON（性能） |
| L4 | LOW | `sql/mod.rs:47` | `$ph1..$ph5` 只有 5 个占位符槽位，未来扩展受限 |
| L5 | LOW | `sql/mod.rs:498-500` | `i64 as u32` 截断 token 计数（需 40 亿+） |
| L6 | LOW | `sql/mod.rs:487` | `Option<String>` for `NOT NULL DEFAULT ''` 列 |
| L7 | LOW | `mysql.rs:115-117` | `let _ =` 吞掉所有 index 创建错误 |
| L8 | LOW | `mongo.rs:321-333` | `]` 未在 regex 中转义 |
| L9 | LOW | `mongo.rs:139` | 大量 sessions 时 `$nin` 性能退化 |
| L10 | LOW | `mongo.rs:205,220` | `usize→i64` / `i64→usize` 转换溢出 |
| L11 | LOW | `mongo.rs:211-212` | 畸形 state 字段静默回退 Default |
| L12 | LOW | `redis.rs:278-287` | limit 计算 `usize→isize` 回绕 |
| L13 | LOW | `redis.rs:303-348` | search 客户端加载全部数据（未文档化） |
| L14 | LOW | `session.rs:462-467` | `now_secs()` 在时钟错误时返回 0 |
| L15 | LOW | `session.rs:304-306` | export 中无效时间戳显示空字符串 |
| L16 | LOW | StatsRepo | mongo/redis 批量操作 N+1 查询 |

---

## 4. 后端能力矩阵

| 能力 | File | SQLite | MySQL | PostgreSQL | Mongo | Redis |
|------|------|--------|-------|------------|-------|-------|
| 编译 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 测试 | ✅ | ✅ (383 pass) | — | — | — | — |
| 基础 CRUD | ✅ | ✅ | ✅ | ⚠️ (F3) | ✅ | ✅ |
| session 写保护 | ❌ (3.6) | ⚠️ (3.2) | ⚠️ (3.2) | ⚠️ (3.2) | ⚠️ (3.13) | ✅ (Lua 原子) |
| 损坏容忍 | ❌ (3.5) | ✅ | ✅ | ❌ (3.3) | ⚠️ (3.15) | ✅ |
| 搜索 | ⚠️ (3.8) | ⚠️ (3.10) | ⚠️ (3.10) | ❌ (3.3) | ⚠️ (3.4) | ❌ (L13) |
| Schema 迁移 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |

---

## 5. 推荐修复路线

| 阶段 | 问题 | 预估工作量 |
|------|------|-----------|
| Phase 4 | 3.1 (SessionManager write-through) + 3.4 (save_session no-op) | 中 |
| Phase 4 | 3.2 (UNIQUE seq 约束) + 3.3 (PG TEXT payload) | 中 |
| Phase 4 | 3.5 (File unwrap_or_default) + 3.7 (I/O 截断) + 3.8 (search 不一致) | 中 |
| Phase 5 | 3.6 (Session TOCTOU lock) + 3.9 (Gateway 绕过) | 中 |
| Phase 5 | 3.10 (search N+1) + 3.12 (批量操作) | 大 |
| Phase 6 | 3.11 (.bak 清理) + 3.13 (Mongo 事务) + 3.14 (孤儿数据) | 小-中 |
| Backlog | L1-L16 (低优先级) | 小 |
