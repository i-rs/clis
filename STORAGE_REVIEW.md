# claw Storage 抽象层审查报告

**审查范围:** `crates/claw/src/storage/` 全部 4 个后端 (file / sql / mongo / redis)、`message/`、`session.rs` 中的消费者代码。

**审查日期:** 2026-06-06

---

## 1. 执行摘要

存储抽象层由 8 个 trait (`SessionRepo`, `MessageLog`, `ApiCacheRepo`, `PlanStepsRepo`, `MemoryRepo`, `StatsRepo`, `SkillRepo`, `ToolCacheRepo`) + `ClawStorage` 容器构成，共 4 个后端实现。代码总量约 3,500 行。

**整体评价:** 架构方向正确 (trait-based DI、async、可插拔后端)，但存在 **6 个关键级别的正确性问题** (其中 3 个会导致数据丢失) 和显著的性能/可扩展性隐患。**当前 file + SQLite 后端在单用户场景下基本可用**；MySQL/PostgreSQL 后端尚未达到生产可用状态。

---

## 2. 问题汇总表 (按严重性排序)

| #  | 严重性 | 类别     | 问题                                          | 影响                                | 位置                                    |
|----|--------|----------|-----------------------------------------------|-------------------------------------|-----------------------------------------|
| 1  | 🔴 关键 | 正确性   | PostgreSQL `?` 占位符完全失效                  | PG 后端所有 SQL 执行都会报错          | `sql/mod.rs:11-14`, `sql/postgres.rs`   |
| 2  | 🔴 关键 | 正确性   | SQL `MAX(seq)+1` 并发竞争 (MySQL/PG)          | 并发追加消息会产生重复 seq，消息覆盖   | `sql/mod.rs:113-138`                    |
| 3  | 🔴 关键 | 正确性   | SQL `load` 查询逻辑错误 + 整数下溢             | 指定 limit 时返回错误数量的消息        | `sql/mod.rs:148-158`                    |
| 4  | 🔴 关键 | 数据丢失 | `index.json` 损坏时静默清空                    | 所有会话元数据永久丢失                | `file.rs:126-140`                       |
| 5  | 🔴 关键 | 数据丢失 | `StatsRepo::prune` 缺锁导致竞争                | 并发 upsert 的数据被 prune 丢失       | `file.rs:344-389`                       |
| 6  | 🔴 关键 | 一致性   | `append_new_messages` 不更新 `message_count`   | `SessionMeta.message_count` 漂移      | `session.rs:367-385`                    |
| 7  | 🟠 高   | 性能     | `SessionRepo::save_all` 全量替换               | 每次更新重写整个 index.json           | `storage/mod.rs:89`, 全后端              |
| 8  | 🟠 高   | 性能     | `SessionRepo` 默认 `get/delete/count` 用 load_all | SQL/Mongo/Redis 下载全部会话元数据   | `storage/mod.rs:92-104`                 |
| 9  | 🟠 高   | 性能     | 搜索将全部消息加载到内存                       | 大型会话搜索内存激增                  | 全后端 `search()`                        |
| 10 | 🟠 高   | 性能     | `SessionManager` 全量 sync-over-async          | 网络后端阻塞 TUI 事件循环             | `session.rs:118,202,270,...`            |
| 11 | 🟡 中   | 设计     | `SCHEMA_VERSION=1` 无迁移路径                  | 未来 schema 升级时旧记录被丢弃         | `message/mod.rs:7`                      |
| 12 | 🟡 中   | 设计     | `Arc<dyn>` vs `Box<dyn>` 不一致                | API 一致性差                          | `storage/mod.rs:234-241`                |
| 13 | 🟡 中   | 设计     | `seq` 类型不一致: u64 / i64 / f64              | 类型转换散落各处，溢出风险             | `message/mod.rs`, sql columns           |
| 14 | 🟡 中   | 设计     | 全局静态 Mutex 序列化跨会话 I/O                | file 后端并发性受限                   | `file.rs:24-25`                         |
| 15 | 🟡 中   | 耦合     | `FileMessageLogStore::search` 直接读 index.json | 违反 trait 抽象，无法替换 SessionRepo | `file.rs:687-696`                       |
| 16 | 🟡 中   | 可维护   | search 逻辑重复 (~100 行 × 4 后端)             | 修一个 bug 要改 4 处                  | 全后端                                   |
| 17 | 🟢 低   | 代码质量 | `clippy::lines_filter_map_ok` 违规             | clippy 不清洁                         | `file.rs` 多处                           |
| 18 | 🟢 低   | 代码质量 | `unwrap_or_default()` 吞掉解析错误             | 用户不知道数据已损坏                  | `file.rs:186-191, 692-693` 等           |
| 19 | 🟢 低   | 安全     | Mongo regex 转义不完整                         | 特殊字符可能导致非预期匹配            | `mongo.rs:289-292`                      |
| 20 | 🟢 低   | 安全     | Redis ZSET score 碰撞导致覆盖                  | seq 重复时 ZADD 静默覆盖旧消息        | `redis.rs:append_batch`                 |

---

## 3. 详细发现

### 3.1 🔴 关键: PostgreSQL `?` 占位符完全失效

**位置:** `sql/mod.rs:11-14` (文档承认), `sql/postgres.rs:162-166`

**问题:** PostgreSQL 协议要求 `$1, $2, $3...` 风格的占位符。`define_sql_stores!` 宏在所有 SQL 中使用 `?`，包括 ON CONFLICT 等 PG 特有语法。PG 后端在第一次执行任何查询时就会报错。

**影响:** PG 后端完全不可用。

**修复方向:**
- 方案 A (推荐): 在 `define_sql_stores!` 宏中添加 dialect 参数，每个 dialect 自行生成 SQL
- 方案 B: 用 sqlx 的 `query!` 宏在编译期绑定到具体 Pool 类型
- 方案 C: 在 PG backend 中用正则替换 `?` → `$N` (不推荐，影响性能)

---

### 3.2 🔴 关键: SQL `MAX(seq)+1` 并发竞争

**位置:** `sql/mod.rs:113-138`

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

**SQLite 安全原因:** SQLite 的写锁是数据库级别的，整个事务期间数据库被锁定。但这也会导致并发性极差。

**影响:** 多 agent 并发追加消息时，消息 seq 重复，后续 `load` 返回的顺序错乱，甚至 ZSET 场景下 (Redis 也有此问题) 数据被覆盖。

**修复方向:**
- 添加 `SELECT ... FOR UPDATE` (MySQL/PG)
- 或使用数据库自增序列 (PG `BIGSERIAL`, MySQL `AUTO_INCREMENT`，但需要单 session 内连续)
- 或使用应用层 advisory lock (`SELECT pg_advisory_xact_lock(hashtext($session_id))`)

---

### 3.3 🔴 关键: SQL `load` 查询逻辑错误

**位置:** `sql/mod.rs:148-158`

**代码:**
```rust
let safe_limit = (limit.min(i64::MAX as usize)) as i64;
let rows = sqlx::query_as(
    "SELECT payload FROM message_log WHERE session_id = ? \
     AND seq > (SELECT COALESCE(MAX(seq), 0) FROM message_log WHERE session_id = ?) - ? \
     ORDER BY seq ASC",
).bind(session_id).bind(session_id).bind(safe_limit).fetch_all(...).await?;
```

**问题 1:** 当 `limit >= MAX(seq)` 时，`MAX(seq) - limit` 变为负数，`seq > 负数` 匹配所有行 → 返回全部消息，而非最后 N 条。

**问题 2:** 当 `limit = usize::MAX` (调用方"取全部"的约定) 时，`safe_limit = i64::MAX`，`MAX(seq) - i64::MAX` 在 i64 中下溢为负数 → 返回全部行 (碰巧"正确"但语义不对)。

**问题 3:** seq 有间隙时 (例如删除了某些消息)，`MAX(seq) - limit` 不会返回 limit 条记录。

**正确写法:**
```sql
SELECT payload FROM (
    SELECT payload FROM message_log
    WHERE session_id = ?
    ORDER BY seq DESC
    LIMIT ?
) sub
ORDER BY seq ASC
```

**影响:** 调用方传入 `max_messages` 时，SQL 后端可能返回超过该数量的消息，导致上下文超出 LLM 限制。

---

### 3.4 🔴 关键: `index.json` 损坏时静默清空

**位置:** `file.rs:126-140`

**代码:**
```rust
async fn load_all(&self) -> anyhow::Result<Vec<SessionMeta>> {
    blocking(move || {
        let content = std::fs::read_to_string(&path)?;
        serde_json::from_str(&content)
            .inspect_err(|e| tracing::error!("index.json 损坏: {}", e))
            .or_else(|_| Ok(Vec::new()))   // ← 损坏时返回空 Vec
    }).await
}
```

**问题:** 如果 `index.json` 因任何原因 (磁盘满、崩溃、并发写) 损坏，`load_all` 返回空 Vec。下次 `save_all` (例如创建新会话) 会用空列表 + 新会话覆盖文件 → **所有历史会话元数据永久丢失**。消息日志文件 (`.jsonl`) 仍然存在，但无法被关联到任何 session。

**影响:** 用户的所有会话列表瞬间清空，且无法通过 UI 恢复。

**修复方向:**
1. `load_all` 损坏时应返回 `Err` 而非空 Vec
2. 在覆盖前对旧文件做备份 (`.bak`)
3. 提供 `rebuild-index` 命令从 `.jsonl` 文件重建 `index.json`

---

### 3.5 🔴 关键: `StatsRepo::prune` 缺锁

**位置:** `file.rs:344-389`

**问题:** `prune` 读取整个 `stats.jsonl`、过滤后重写文件，但**没有持有 `STATS_LOCK`**。对比 `upsert_batch` (line 320) 正确持锁。

**竞争场景:**
```
Thread A (prune):                     Thread B (upsert_batch):
  read stats.jsonl → [old, new]       
  filter → keep [new]                 
                                       acquire STATS_LOCK
                                       append [newer] → stats.jsonl
                                       release STATS_LOCK
  write [new] → stats.jsonl (覆盖)
```
Thread B 追加的 `newer` 记录被 Thread A 的覆盖丢失。

**影响:** 统计数据随机丢失。

**修复:** 在 `prune` 的 `blocking` 闭包开头加 `let _guard = lock_guard(&STATS_LOCK);`。

---

### 3.6 🔴 关键: `append_new_messages` 不更新 `message_count`

**位置:** `session.rs:367-385`

**代码:**
```rust
pub fn append_new_messages(&mut self, session_id: &str, messages: &[Message]) {
    let cursor = self.saved_cursors.get(session_id).copied().unwrap_or(0);
    let new_msgs = &messages[cursor..];
    // ... append_batch ...
    match result {
        Ok(()) => {
            self.saved_cursors.insert(session_id.to_string(), messages.len());
            // ← 没有更新 SessionMeta.message_count
        }
        Err(e) => tracing::error!("..."),
    }
}
```

**问题:** `SessionMeta.message_count` 字段存在但从未被 `append_new_messages` 更新。该字段会逐渐小于实际消息数。

**影响:** UI 显示的未读/消息数不准；依赖该字段的逻辑 (如压缩触发条件) 失效。

**修复:** 在 `Ok(())` 分支更新 `SessionMeta.message_count += new_msgs.len()` 并 `save_index()`。

---

### 3.7 🟠 高: `SessionRepo::save_all` 全量替换

**位置:** `storage/mod.rs:89`, 全部 4 个后端

**问题:** trait 只暴露 `save_all(&[SessionMeta])` — 每次更新任何会话都要重写整个索引。在 file 后端，这意味着每次发消息都会触发 `index.json` 完整序列化 + 原子替换。

**file 后端示例链:**
```
用户发消息 → session.update_at = now() → save_index() → 重写 index.json
```

**影响:**
- 1000 个会话时，每次更新 ~50KB JSON 序列化 + fsync
- SSD 写入放大
- 多个并发会话更新互相阻塞 (STATS_LOCK)

**修复方向:** trait 添加 `async fn upsert(&self, session: &SessionMeta)` 和 `async fn delete(&self, id: &str)` 方法，让 SQL/Mongo/Redis 用单行操作实现。file 后端仍可 fallback 到 `save_all`。

---

### 3.8 🟠 高: `SessionRepo` 默认实现使用 `load_all`

**位置:** `storage/mod.rs:92-104`

**代码:**
```rust
async fn get(&self, id: &str) -> Result<Option<SessionMeta>> {
    Ok(self.load_all().await?.into_iter().find(|s| s.id == id))
}
async fn delete(&self, id: &str) -> Result<()> {
    let mut sessions = self.load_all().await?;
    sessions.retain(|s| s.id != id);
    self.save_all(&sessions).await
}
```

**问题:** SQL/Mongo/Redis 都有按 ID 操作的能力，但 trait 默认实现强制下载全部会话再过滤。这完全抵消了数据库后端的优势。

**影响:** 每次删除一个会话都要传输所有会话元数据到客户端。

**修复:** 将 `get`/`delete`/`count` 改为必需方法 (无默认实现)，强制每个后端高效实现。

---

### 3.9 🟠 高: 搜索全量加载到内存

**位置:** 全部 4 个后端的 `search()` 方法

**问题:** 搜索流程是:
1. 找出所有候选 session IDs
2. **对每个 session，加载全部消息到 `Vec<serde_json::Value>`** (file: `Vec<StoredRecord>`, sql: `Vec<(i64, String)>`, mongo/redis: `Vec<Document>`)
3. 在内存中线性扫描

**示例 (mongo.rs:325-340):**
```rust
let cursor = collection.find(doc!{"session_id": sid}).sort(doc!{"seq":1}).await?;
let docs: Vec<Document> = cursor.try_collect().await?;
let records: Vec<serde_json::Value> = docs.into_iter()
    .filter_map(|d| serde_json::from_str(d.get_str("payload")?).ok())
    .collect();
```

**影响:** 一个 10,000 条消息的会话 + 100 个会话 = 100 万条消息加载到内存中做 substring 匹配。

**修复方向:**
- 短期: 在 SQL/Mongo 层做 `WHERE payload LIKE '%query%'` 过滤，只返回匹配行
- 中期: 全文索引 (PG `tsvector`, Mongo text index, Redis `FT.SEARCH`)
- 长期: 语义搜索 (已有 `semantic.rs` TF-IDF，可复用)

---

### 3.10 🟠 高: `SessionManager` 全量 sync-over-async

**位置:** `session.rs` 多处 (118, 202, 270, 280, 356, 377, 398, 409, 426)

**实现 (`utils.rs:16-21`):**
```rust
static SHARED_RUNTIME: LazyLock<tokio::runtime::Runtime> = ...;
pub fn sync_block_on<F: Future>(f: F) -> F::Output {
    match tokio::runtime::Handle::try_current() {
        Ok(_) => tokio::task::block_in_place(|| SHARED_RUNTIME.block_on(f)),
        Err(_) => SHARED_RUNTIME.block_on(f),
    }
}
```

**问题:** `SessionManager` 的所有方法都是同步的 (`&mut self`, no async)，但底层存储是 async。每次调用都通过 `sync_block_on` 桥接。

**场景 1 (file 后端):** 影响较小 — `blocking()` 把 I/O 推到 tokio blocking thread pool，sync_block_on 只是等待。

**场景 2 (网络后端):** `block_in_place` 会把当前 worker thread 转为 parking 模式，期间该 thread 不处理任何其他任务。在高频消息追加场景 (例如工具循环连续 10 个调用) 下，TUI 渲染会卡顿。

**场景 3 (已在 tokio context):** `block_in_place` 在 `current_thread` runtime 下会 panic。TUI 用的 `tokio::runtime::Runtime::new()` 默认是 multi-thread，所以目前安全。但如果未来改为 current_thread (例如做单核嵌入式)，就会崩溃。

**修复方向:** 将 `SessionManager` 改为 async，或用 channel 把存储操作推到独立 actor。

---

### 3.11 🟡 中: `SCHEMA_VERSION=1` 无迁移路径

**位置:** `message/mod.rs:7`

**问题:** `StoredRecord.schema_v = 1`，`to_message()` 返回 `Option<Message>`。如果未来 schema_v=2 的记录被旧代码读取，会得到 `None`，消息被静默丢弃。

**修复:** 在 `StoredRecord` 上添加 `try_from_v1`/`try_from_v2` 方法，存储层在 load 时检测版本并触发迁移。

---

### 3.12 🟡 中: `Arc<dyn>` vs `Box<dyn>` 不一致

**位置:** `storage/mod.rs:234-241`

```rust
pub struct ClawStorage {
    pub sessions:    Box<dyn SessionRepo>,
    pub message_log: Arc<dyn MessageLog>,  // ← 唯一一个 Arc
    pub api_cache:   Box<dyn ApiCacheRepo>,
    ...
}
```

**原因:** `SessionManager::message_log()` (session.rs:361) 返回 `Arc<dyn MessageLog>` 供外部追加 (避免 `&self` 生命周期限制)。

**影响:** API 不一致，使用者需要记忆哪个是 `Box` 哪个是 `Arc`。

**修复:** 全部改为 `Arc<dyn>`，或 `SessionManager` 改为持有 `ClawStorage` 的引用而非克隆 `message_log`。

---

### 3.13 🟡 中: `seq` 类型不一致

**位置:** 多处

| 位置               | 类型     |
|--------------------|----------|
| `StoredRecord.seq` | `u64`    |
| SQL `seq` 列       | `BIGINT` / `i64` |
| Mongo `seq` 字段   | `i64`    |
| Redis ZSET score   | `f64`    |
| `SessionRow.message_count` | `i64` → `as usize` |

**影响:** `u64 as i64` 在 `u64 > i64::MAX` 时静默截断；`i64 as u64` 在负数时变成巨大正数。当前 seq 不会到那么大，但类型系统没有保护。

**修复:** 统一用 `u64` 或 `i64`，在边界做 checked cast。

---

### 3.14 🟡 中: 全局静态 Mutex

**位置:** `file.rs:24-25`

```rust
static MESSAGES_LOCK: Mutex<()> = Mutex::new(());
static STATS_LOCK: Mutex<()> = Mutex::new(());
```

**问题:** 进程级别的全局锁，序列化所有会话的 message 追加操作。如果两个 agent 同时对话，它们的消息追加互相阻塞。

**影响:** 多 agent 并发场景下性能下降。

**修复:** 改为 per-session 的锁 (例如 `DashMap<String, Mutex<()>>`)。

---

### 3.15 🟡 中: File search 直接读 index.json

**位置:** `file.rs:687-696`

```rust
let index_path = claw_dir.join("index.json");
// NOTE: search reads index.json directly — coupled to FileSessionStore format.
let sessions = std::fs::read_to_string(&index_path)...
```

**问题:** `FileMessageLogStore::search` 直接读 `index.json`，假设了 `FileSessionStore` 的存储格式。如果未来 file 后端改用多文件或 SQLite 存 sessions，search 就会失效。

**修复:** search 应通过 `SessionRepo::load_all()` 获取 session 列表。需要重构 trait 或在 `FileMessageLogStore` 中注入 `Arc<dyn SessionRepo>`。

---

### 3.16 🟡 中: search 逻辑重复

**位置:** 4 个后端各有一份 ~100 行近乎相同的 search 代码。

**重复片段:**
- 提取 `payload["text"]` / `payload["name"]`
- 大小写不敏感匹配
- 200 字符摘要截取
- 前后上下文 (2 前 1 后)
- `SearchResult` 构造

**修复:** 提取为 `fn build_search_result(records: &[(usize, Value)], q: &str, ...) -> Vec<SearchResult>`，后端只需提供"找出包含 query 的 session IDs"和"加载该 session 的全部消息"。

---

### 3.17-3.20 🟢 低 (略)

- **clippy 违规:** `file.rs` 多处 `lines().filter_map(|l| l.ok())` 触发 `clippy::lines_filter_map_ok`。
- **静默吞错:** `unwrap_or_default()` 在 api_cache load、index.json 解析等处隐藏了真正的 I/O / 解析错误。
- **Mongo regex 转义不全:** `mongo.rs:289-292` 只转义了部分元字符 (`.` `*` `+` `?` 等)，漏掉 `]` `}` `/`。
- **Redis ZSET score 碰撞:** 如果 seq 相同 (由于 #3.2 的竞争)，`ZADD` 会用新 member 覆盖旧 member 的 score，但因为 score+member 相同，旧消息被替换。需要确保 seq 单调递增。

---

## 4. 后端能力矩阵

| 能力                | File      | SQLite    | MySQL     | PostgreSQL | Mongo     | Redis     |
|---------------------|-----------|-----------|-----------|------------|-----------|-----------|
| 基础 CRUD           | ✅        | ✅        | ✅        | ❌ (#1)    | ✅        | ✅        |
| 原子 seq            | ✅ (锁)   | ✅ (锁)   | ❌ (#2)   | ❌ (#2)    | ✅ (原子) | ✅ (INCR) |
| 按 ID 查会话        | ❌ (#8)   | ❌ (#8)   | ❌ (#8)   | ❌ (#8)    | ❌ (#8)   | ❌ (#8)   |
| 单会话更新          | ❌ (#7)   | ❌ (#7)   | ❌ (#7)   | ❌ (#7)    | ❌ (#7)   | ❌ (#7)   |
| 流式搜索            | ❌ (#9)   | ❌ (#9)   | ❌ (#9)   | ❌ (#9)    | ❌ (#9)   | ❌ (#9)   |
| 搜索逻辑共享        | ❌ (#16)  | ❌ (#16)  | ❌ (#16)  | ❌ (#16)   | ❌ (#16)  | ❌ (#16)  |
| Schema 迁移         | ❌ (#11)  | ❌ (#11)  | ❌ (#11)  | ❌ (#11)   | ❌ (#11)  | ❌ (#11)  |

---

## 5. 建议修复优先级

### Phase 1 — 关键正确性 (建议立即修复)

1. **#4** index.json 损坏保护 — 加备份 + 返回 Err
2. **#5** StatsRepo::prune 加锁
3. **#6** append_new_messages 更新 message_count
4. **#3** SQL load 查询改用子查询 + LIMIT

### Phase 2 — 后端可用性 (1-2 周)

5. **#1** PostgreSQL 占位符支持
6. **#2** SQL append_batch 用 `FOR UPDATE` 或 advisory lock
7. **#7 + #8** SessionRepo trait 重构: 添加 `upsert` / `delete_one` / `get_one` 必需方法

### Phase 3 — 性能与可扩展性 (2-4 周)

8. **#9** 搜索下推到数据库 (LIKE / 全文索引)
9. **#10** SessionManager async 化 (或 actor 化)
10. **#16** 提取共享 search 逻辑

### Phase 4 — 代码质量 (持续)

11. **#11-#15** 设计层面改进
12. **#17-#20** Clippy + 错误处理

---

## 6. 审查覆盖文件清单

| 文件                                 | 行数  | 审查状态 |
|--------------------------------------|-------|----------|
| `storage/mod.rs`                     | 250   | ✅ 完整  |
| `storage/file.rs`                    | 1295  | ✅ 完整  |
| `storage/sql/mod.rs`                 | 555   | ✅ 完整  |
| `storage/sql/sqlite.rs`              | 154   | ✅ 完整  |
| `storage/sql/mysql.rs`               | 176   | ✅ 完整  |
| `storage/sql/postgres.rs`            | 167   | ✅ 完整  |
| `storage/mongo.rs`                   | 943   | ✅ 完整  |
| `storage/redis.rs`                   | 777   | ✅ 完整  |
| `message/mod.rs`                     | 162   | ✅ 完整  |
| `session.rs`                         | 712   | ✅ 完整 (消费者视角) |
| `utils.rs` (sync_block_on)           | ~20   | ✅ 完整  |
| `core/mod.rs` (block_on)             | ~5    | ✅ 完整  |
