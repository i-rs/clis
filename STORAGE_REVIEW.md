# claw Storage 抽象层审查报告 (v2)

**审查范围:** `crates/claw/src/storage/` 全部 4 个后端 (file / sql / mongo / redis)、`message/`、`session.rs` 中的消费者代码。

**审查日期:** 2026-06-06 (初版) / 2026-06-06 (v2, 基于 commit `a3581b29`)

**v2 变更说明:** 基于 Phase 1 修复 (commit `bbec4825`) 后的代码重新审查。6 个关键问题中 4 个已修复、1 个修复不完整、1 个未触及；同时发现 3 个修复引入的新问题。

---

## 1. 执行摘要

存储抽象层由 8 个 trait + `ClawStorage` 容器构成，共 4 个后端实现，代码总量约 3,700 行。

**Phase 1 修复后整体状态:**
- ✅ **4 个关键 bug 已修复:** SQL load 查询、index.json 备份、prune 加锁、message_count 漂移
- ⚠️ **1 个关键修复不完整:** index.json 损坏保护在存储层已修，但 `SessionManager` 消费者仍用 `unwrap_or_default()` 吞掉错误
- ❌ **2 个关键问题未触及:** PostgreSQL 占位符、SQL seq 竞争
- 🔸 **3 个新问题由修复引入:** Redis 死代码 + 错误吞噬、SessionManager 未利用新 trait 方法、不必要的 clone

**后端可用性:**
- File + SQLite: 单用户可用 ✅
- MongoDB: 基本可用 ✅
- Redis: 基本可用，但有死代码需清理 ⚠️
- MySQL: 并发追加有竞争 ⚠️
- PostgreSQL: 完全不可用 ❌

---

## 2. 问题汇总表 (按严重性排序)

### 2.1 当前仍存在的问题

| #  | 严重性 | 类别     | 问题                                          | 状态       | 位置                                    |
|----|--------|----------|-----------------------------------------------|------------|-----------------------------------------|
| 1  | 🔴 关键 | 正确性   | PostgreSQL `?` 占位符完全失效                  | ✅ 已修复   | `sql/mod.rs`, `sql/postgres.rs`        |
| 2  | 🔴 关键 | 正确性   | SQL `MAX(seq)+1` 并发竞争 (MySQL/PG)          | ✅ 已修复   | `sql/mod.rs`, dialect files            |
| 4R | 🔴 关键 | 数据丢失 | index.json 损坏保护 — 消费者仍吞错 (回归)      | ✅ 已修复   | `session.rs` (Phase 1.5)               |
| 9  | 🟠 高   | 性能     | 搜索将全部消息加载到内存                       | 未修复     | 全后端 `search()`                        |
| 10 | 🟠 高   | 性能     | `SessionManager` 全量 sync-over-async          | 未修复     | `session.rs:118,202,379,410,419,438`   |
| 11 | 🟡 中   | 设计     | `SCHEMA_VERSION=1` 无迁移路径                  | 未修复     | `message/mod.rs:7`                      |
| 12 | 🟡 中   | 设计     | `Arc<dyn>` vs `Box<dyn>` 不一致                | 未修复     | `storage/mod.rs:231-239`                |
| 13 | 🟡 中   | 设计     | `seq` 类型不一致: u64 / i64 / f64              | 未修复     | `message/mod.rs`, sql columns           |
| 14 | 🟡 中   | 设计     | 全局静态 Mutex 序列化跨会话 I/O                | 未修复     | `file.rs:24-25`                         |
| 15 | 🟡 中   | 耦合     | `FileMessageLogStore::search` 直接读 index.json | 未修复    | `file.rs:713-722`                       |
| 16 | 🟡 中   | 可维护   | search 逻辑重复 (~100 行 × 4 后端)             | ✅ 已修复   | 全后端 → `scan_records_for_query`      |
| 19 | 🟢 低   | 安全     | Mongo regex 转义不完整                         | 未修复     | `mongo.rs:289-292`                      |
| 20 | 🟢 低   | 安全     | Redis ZSET score 碰撞导致覆盖                  | 未修复     | `redis.rs:append_batch`                 |

### 2.2 修复引入的新问题

| #  | 严重性 | 类别     | 问题                                          | 位置                                    |
|----|--------|----------|-----------------------------------------------|-----------------------------------------|
| N1 | 🟠 高   | 代码质量 | Redis `get_one` 用 `.ok()` 吞 JSON 解析错误     | ✅ 已修复   | `redis.rs` (Phase 1.5)                 |
| N2 | 🟠 高   | 死代码   | Redis 残留旧 `get`/`delete` 方法 (非 trait 成员) | ✅ 已修复   | `redis.rs` (Phase 1.5)                 |
| N3 | 🟡 中   | 性能     | `SessionManager` 未使用新 `upsert`/`delete_one` 方法 | ✅ 已修复   | `session.rs` (Phase 2)                 |
| N4 | 🟢 低   | 代码质量 | `append_new_messages` 不必要的 `new_msgs.clone()` | ✅ 已修复   | `session.rs` (Phase 1.5)               |
| N5 | 🟢 低   | 格式     | SQL `get_one` 查询字符串含大量多余空格          | ✅ 已修复   | `sql/mod.rs` (Phase 1.5)               |

### 2.3 已修复的问题

| #  | 问题                                  | 修复 commit  | 验证状态 |
|----|--------------------------------------|--------------|----------|
| 3  | SQL `load` 查询逻辑错误 + 整数下溢     | `bbec4825`   | ✅ 已验证: 子查询 + `LIMIT` + `ORDER BY` |
| 4  | `index.json` 损坏时静默清空 (存储层)   | `bbec4825`   | ⚠️ 存储层已修，消费者未修 → 见 #4R |
| 5  | `StatsRepo::prune` 缺锁导致竞争        | `bbec4825`   | ✅ 已验证: `file.rs:377` 持有 `STATS_LOCK` |
| 6  | `append_new_messages` 不更新 `message_count` | `a3581b29` (后续修补) | ✅ 已验证: `session.rs:384-393` |
| 7  | `SessionRepo::save_all` 全量替换 (trait 层) | `bbec4825`   | ✅ trait 已添加 `upsert`/`delete_one`/`get_one`/`count` |
| 8  | `SessionRepo` 默认实现使用 `load_all`  | `bbec4825`   | ✅ 默认实现已移除，改为必需方法 |

---

## 3. 详细发现

### 3.1 🔴 关键: PostgreSQL `?` 占位符完全失效 (未修复)

**位置:** `sql/mod.rs:14-17` (文档承认), `sql/postgres.rs:162-166`

**问题:** PostgreSQL 协议要求 `$1, $2, $3...` 风格的占位符。`define_sql_stores!` 宏在所有 SQL 中使用 `?`，包括 `ON CONFLICT` 等 PG 特有语法。PG 后端在第一次执行任何查询时就会报错。

**影响:** PG 后端完全不可用。

**修复方向:**
- 方案 A (推荐): 在 `define_sql_stores!` 宏中添加 dialect 参数，每个 dialect 自行生成 SQL
- 方案 B: 用 sqlx 的 `query!` 宏在编译期绑定到具体 Pool 类型
- 方案 C: 在 PG backend 中用正则替换 `?` → `$N` (不推荐，影响性能)

---

### 3.2 🔴 关键: SQL `MAX(seq)+1` 并发竞争 (未修复)

**位置:** `sql/mod.rs:146-172`

**代码:**
```rust
let mut tx = self.db.pool.begin().await?;
let next_seq: i64 = sqlx::query_scalar(
    "SELECT COALESCE(MAX(seq), 0) FROM message_log WHERE session_id = ?",
).fetch_one(&mut *tx).await?;
let mut seq = next_seq + 1;
for msg in messages { /* INSERT ... */ }
tx.commit().await?;
```

**问题:** 经典的 read-modify-write 反模式。在 MySQL (READ COMMITTED) 和 PostgreSQL (READ COMMITTED) 默认隔离级别下，两个并发事务可能读到相同的 `MAX(seq)`，导致重复 seq。

**SQLite 安全原因:** SQLite 的写锁是数据库级别的，整个事务期间数据库被锁定。

**影响:** 多 agent 并发追加消息时，消息 seq 重复，后续 `load` 返回的顺序错乱。

**修复方向:**
- 添加 `SELECT ... FOR UPDATE` (MySQL/PG)
- 或使用数据库自增序列 (PG `BIGSERIAL`, MySQL `AUTO_INCREMENT`)
- 或使用应用层 advisory lock (`SELECT pg_advisory_xact_lock(hashtext($session_id))`)

---

### 3.4R 🔴 关键: index.json 损坏保护 — 消费者仍吞错 (修复不完整)

**位置:** `session.rs:118-120`

**背景:** 存储层 `FileSessionStore::load_all` (`file.rs:133-137`) 已正确改为返回 `Err`。但消费者 `SessionManager::with_storage` 仍然吞掉错误：

```rust
pub fn with_storage(storage: Arc<ClawStorage>) -> Self {
    let sessions = crate::utils::sync_block_on(async {
        storage.sessions.load_all().await.unwrap_or_default()  // ← 仍然吞错
    });
    ...
}
```

**问题:** 如果 `index.json` 损坏，`load_all` 返回 `Err`，`unwrap_or_default()` 将其转为空 Vec。`SessionManager` 以空列表启动，随后第一次 `save_index()` (例如创建新会话) 调用 `save_all`，覆盖 `index.json` (虽然有 `.bak` 备份，但主文件已丢失所有会话)。

**影响:** 存储层的修复 (#4) 被消费者的 `unwrap_or_default()` 绕过。用户仍会丢失全部会话列表，只是现在有了 `.bak` 文件作为最后手段。

**修复方向:**
```rust
let sessions = crate::utils::sync_block_on(async {
    storage.sessions.load_all().await
}).unwrap_or_else(|e| {
    tracing::error!("加载会话列表失败: {}, 尝试从 .bak 恢复", e);
    // 尝试从 index.json.bak 恢复，或返回空列表但不立即覆盖
    Vec::new()  // 标记为"恢复模式"，避免覆盖
});
```

或更好的方案：`with_storage` 返回 `Result<Self>`，让调用方决定如何处理错误。

---

### 3.9 🟠 高: 搜索全量加载到内存 (未修复)

**位置:** 全部 4 个后端的 `search()` 方法

**问题:** 搜索流程对每个候选 session 加载全部消息到 `Vec`，然后在内存中线性扫描。

**影响:** 100 个会话 × 10,000 条消息 = 100 万条消息加载到内存中做 substring 匹配。

**修复方向:**
- 短期: SQL `WHERE payload LIKE '%query%'` / Mongo `$regex` 只返回匹配行
- 中期: 全文索引 (PG `tsvector`, Mongo text index, Redis `FT.SEARCH`)
- 长期: 语义搜索 (已有 `semantic.rs` TF-IDF)

---

### 3.10 🟠 高: SessionManager 全量 sync-over-async (未修复)

**位置:** `session.rs:118, 202, 379, 410, 419, 438`

**问题:** `SessionManager` 的所有方法都是同步的，每次调用通过 `sync_block_on` 桥接 async 存储。

**影响:** 网络后端 (Mongo/Redis/MySQL/PG) 下，`block_in_place` 会阻塞 tokio worker thread。高频消息追加场景下 TUI 渲染会卡顿。

**修复方向:** SessionManager async 化，或用 channel 把存储操作推到独立 actor。

---

### 3.N1 🟠 高 (新): Redis `get_one` 吞 JSON 解析错误

**位置:** `redis.rs:128-132`

**代码:**
```rust
async fn get_one(&self, id: &str) -> anyhow::Result<Option<SessionMeta>> {
    let json: Option<String> = redis::cmd("HGET")...query_async(...).await?;
    Ok(json.and_then(|j| serde_json::from_str(&j).ok()))  // ← .ok() 吞错
}
```

**对比旧代码 (`redis.rs:162-170`, 现为死代码):**
```rust
async fn get(&self, id: &str) -> anyhow::Result<Option<SessionMeta>> {
    let json: Option<String> = ...;
    match json {
        Some(j) => Ok(Some(serde_json::from_str(&j)?)),  // ← 正确传播错误
        None => Ok(None),
    }
}
```

**影响:** 如果 Redis 中的 session JSON 损坏，`get_one` 静默返回 `None`，调用方以为会话不存在。

**修复:** 用 `serde_json::from_str(&j).map_err(|e| anyhow::anyhow!("session JSON 解析失败: {}", e)).map(Some)` 或直接 `?`。

---

### 3.N2 🟠 高 (新): Redis 残留旧 `get`/`delete` 方法

**位置:** `redis.rs:162-180`

**问题:** `SessionRepo` trait 已移除 `get`/`delete`，改为 `get_one`/`delete_one`。但 `RedisSessionStore` 仍然保留了旧的 `get`/`delete` 作为固有方法 (inherent methods)。这些方法不是 trait 成员，编译器不会报错，但它们是**死代码**。

**影响:** 维护负担，且 `get` (传播错误) 比 `get_one` (吞错) 实现更好，容易造成混淆。

**修复:** 删除 `redis.rs:162-180` 的 `get` 和 `delete` 方法。

---

### 3.N3 🟡 中 (新): SessionManager 未使用新 trait 方法

**位置:** `session.rs:434-442` (`save_index` 方法)

**代码:**
```rust
pub(crate) fn save_index(&self) {
    let storage = self.storage.clone();
    let sessions = self.sessions.clone();
    // 仍然用 save_all 重写整个索引
    crate::utils::sync_block_on(async move { storage.sessions.save_all(&sessions).await })
}
```

**问题:** Phase 1 给 `SessionRepo` trait 添加了 `upsert`/`delete_one` 等单行操作方法，SQL/Mongo/Redis 后端都实现了高效的单行操作。但 `SessionManager` (唯一消费者) 仍然只调用 `load_all` + `save_all`，完全忽略了新方法。

**影响:** 新增的 4 个 trait 方法 + 4 个后端 × 4 个实现 = 16 个新方法未被使用。性能改进为零。

**修复方向:** 重构 `SessionManager`:
- `create_session` → 调用 `upsert` 而非 `save_all`
- `delete_session` → 调用 `delete_one` 而非 `save_all`
- `rename_session` / `transition_state` → 调用 `upsert` 而非 `save_all`
- `append_new_messages` → 已经调了 `save_index`，改为调 `upsert` 只更新单个 session

---

### 3.11-3.16 🟡 中 (未变)

| #  | 问题                                  | 状态     |
|----|--------------------------------------|----------|
| 11 | `SCHEMA_VERSION=1` 无迁移路径         | 未修复   |
| 12 | `Arc<dyn>` vs `Box<dyn>` 不一致       | 未修复   |
| 13 | `seq` 类型不一致: u64 / i64 / f64     | 未修复   |
| 14 | 全局静态 Mutex 序列化跨会话 I/O       | 未修复   |
| 15 | File search 直接读 index.json         | 未修复   |
| 16 | search 逻辑重复 (~100 行 × 4 后端)    | 未修复   |

---

### 3.N4 🟢 低 (新): append_new_messages 不必要的 clone

**位置:** `session.rs:375-377`

**代码:**
```rust
let new_msgs = new_msgs.to_vec();        // clone 1
let append_count = new_msgs.len();
let new_msgs_clone = new_msgs.clone();   // clone 2 (不必要)
let result =
    crate::utils::sync_block_on(async move { log.append_batch(&sid, &new_msgs_clone).await });
```

**问题:** `new_msgs` 在 clone 后不再使用，第二个 clone 是多余的。

**修复:** 直接 move `new_msgs` 进 async block。

---

### 3.N5 🟢 低 (新): SQL get_one 查询字符串多余空格

**位置:** `sql/mod.rs:97`

**代码:**
```rust
"SELECT id, title, agent_id, state, created_at, updated_at, message_count                      FROM sessions WHERE id = ?"
```

**问题:** `message_count` 和 `FROM` 之间有大量空格 (约 20 个)。虽然 SQL 语义不受影响，但看起来像是编辑器粘贴错误。

**修复:** 清理多余空格。

---

## 4. 后端能力矩阵 (Phase 1 后)

| 能力                | File      | SQLite    | MySQL     | PostgreSQL | Mongo     | Redis     |
|---------------------|-----------|-----------|-----------|------------|-----------|-----------|
| 基础 CRUD           | ✅        | ✅        | ✅        | ❌ (#1)    | ✅        | ✅        |
| 原子 seq            | ✅ (锁)   | ✅ (锁)   | ❌ (#2)   | ❌ (#2)    | ✅ (原子) | ✅ (INCR) |
| 按 ID 查会话        | ⚠️ (fallback) | ✅   | ✅        | ❌ (#1)    | ✅        | ⚠️ (N1: 吞错) |
| 单会话 upsert       | ⚠️ (fallback) | ✅   | ✅        | ❌ (#1)    | ✅        | ✅        |
| 单会话 delete       | ⚠️ (fallback) | ✅   | ✅        | ❌ (#1)    | ✅        | ✅        |
| 损坏保护            | ⚠️ (#4R)  | N/A       | N/A       | N/A        | N/A       | N/A       |
| 流式搜索            | ❌ (#9)   | ❌ (#9)   | ❌ (#9)   | ❌ (#9)    | ❌ (#9)   | ❌ (#9)   |
| Schema 迁移         | ❌ (#11)  | ❌ (#11)  | ❌ (#11)  | ❌ (#11)   | ❌ (#11)  | ❌ (#11)  |

**File 后端 "fallback" 含义:** 实现了 `get_one`/`upsert`/`delete_one`/`count`，但内部仍 fallback 到 `load_all` + `save_all`。

---

## 5. 建议修复优先级 (修订版)

### Phase 1.5 — 修复 Phase 1 的遗漏 (立即)

1. **#4R** `SessionManager::with_storage` 不再吞 `load_all` 错误 — 返回 `Result` 或做恢复处理
2. **N1** Redis `get_one` 用 `?` 传播 JSON 解析错误
3. **N2** 删除 Redis 残留的旧 `get`/`delete` 死代码
4. **N5** 清理 SQL `get_one` 多余空格
5. **N4** 移除 `append_new_messages` 多余 clone

### Phase 2 — 后端可用性 (1-2 周)

6. **#1** PostgreSQL `$N` 占位符支持
7. **#2** SQL `append_batch` 用 `FOR UPDATE` 或 advisory lock
8. **N3** `SessionManager` 改用 `upsert`/`delete_one` 取代 `save_all`

### Phase 3 — 性能与可扩展性 (2-4 周)

9. **#9** 搜索下推到数据库 (LIKE / 全文索引)
10. **#10** SessionManager async 化 (或 actor 化)
11. **#16** 提取共享 search 逻辑

### Phase 4 — 设计改进 (持续)

12. **#11** Schema 迁移路径
13. **#12** Arc/Box 统一
14. **#13** seq 类型统一
15. **#14** per-session 锁替代全局 Mutex
16. **#15** File search 解耦 index.json

---

## 6. 审查覆盖文件清单

| 文件                                 | 行数  | 审查状态 |
|--------------------------------------|-------|----------|
| `storage/mod.rs`                     | 248   | ✅ v2 完整 |
| `storage/file.rs`                    | 1326  | ✅ v2 完整 |
| `storage/sql/mod.rs`                 | 594   | ✅ v2 完整 |
| `storage/sql/sqlite.rs`              | 154   | ✅ 未变   |
| `storage/sql/mysql.rs`               | 176   | ✅ 未变   |
| `storage/sql/postgres.rs`            | 167   | ✅ 未变   |
| `storage/mongo.rs`                   | 977   | ✅ v2 完整 |
| `storage/redis.rs`                   | 811   | ✅ v2 完整 |
| `message/mod.rs`                     | 162   | ✅ 未变   |
| `session.rs`                         | 724   | ✅ v2 完整 |
| `utils.rs` (sync_block_on)           | ~20   | ✅ 未变   |
| `core/mod.rs` (block_on)             | ~5    | ✅ 未变   |

---

## 7. 变更日志

| 日期       | 版本 | 变更                                              |
|------------|------|---------------------------------------------------|
| 2026-06-06 | v1   | 初始审查 (20 个问题)                               |
| 2026-06-06 | v2   | Phase 1 后复审: 6 个已修复/部分修复, 5 个新问题发现 |
