# 存储层抽象设计方案

## 1. 当前存储层现状

`~/.i-rs/claw/` 目录下共 **7 个存储域**，全部基于文件 JSON/JSONL：

```
~/.i-rs/claw/
├── index.json                          # SessionManager: Vec<SessionMeta>
├── sessions/
│   ├── {id}.jsonl                      # 消息 (JSONL, 逐行 append)
│   ├── {id}_api.json                   # API 消息缓存
│   └── {id}_plan.json                  # Plan steps
├── agents/{agent_id}/
│   ├── memory.json                     # CrossSessionMemory
│   ├── hot_docs_cache.json             # ToolDocCache
│   └── skills/*.md                     # SkillStore
└── stats/
    └── usage.jsonl                     # TokenRecord (JSONL, 缓冲+锁)

相关源文件:
  session.rs    -> SessionManager     (538 行)
  memory.rs     -> CrossSessionMemory (361 行)
  convstore.rs  -> ConvStore          (158 行)
  stats/mod.rs  -> StatsManager       (322 行)
  stats/store.rs-> JSONL 读写+索引    (266 行)
  skill_store.rs-> SkillStore         (642 行)
  tool_cache.rs -> ToolDocCache       (129 行)
```

| 存储域 | 格式 | 操作模式 | 查询方式 |
|--------|------|---------|---------|
| Session 索引 | JSON 全量读写 | load all → modify → save all | 内存 find |
| 消息 | JSONL append | append 单行 / 全量覆盖 | 全量扫描 |
| API 缓存 | JSON 全量读写 | save / load | key 直读 |
| Memory | JSON 全量读写 | dirty flag + flush | key 直读 |
| Stats | JSONL append | 缓冲批量 append | 时间范围扫描 |
| Skills | .md 文件 | 文件级 CRUD | 目录扫描 |
| ToolCache | JSON 全量读写 | save / load | key 直读 |

**共同特征**：全同步 I/O、`serde_json` 序列化、无事务、无索引（除 stats 的简易 offset index）、无迁移机制。

---

## 2. 设计目标

1. **不改业务代码核心逻辑** — SessionManager / CrossSessionMemory 等的业务方法保持不动
2. **可插拔后端** — 通过 config 切换 file / sqlite / mysql / pg / mongodb
3. **渐进式迁移** — 可先用 file 跑，后期切 SQLite 零配置，再上 MySQL/PG 做多端共享
4. **不引入 ORM 重依赖** — 避免 diesel/sea-orm 的编译时间，用轻量抽象

---

## 3. 核心思路：Repository 模式 + 最小化 trait

每个存储域拆出一个 **Repository trait**，file backend 直接搬现有代码，sql/mongo backend 实现同样的 trait。

```
                    ┌──────────────────┐
                    │    AppCore       │
                    └──┬───────────────┘
                       │ owns
              ┌────────┴────────┐
              ▼                 ▼
      SessionManager      AgentRuntimeStore
      (业务逻辑保留)       (业务逻辑保留)
              │                 │
              ▼                 ▼
      SessionRepo        MemoryRepo / SkillRepo / ...
      (trait)            (traits)
              │                 │
     ┌────────┼─────────┐       │
     ▼        ▼         ▼       ▼
  FileRepo  SqlRepo  MongoRepo  ...
```

### 为什么不用单一的 `StorageBackend` 大 trait？

- **编译单元隔离**：SessionRepo 改了不影响 StatsRepo 的调用者
- **依赖精确**：Stats 需要批量写入能力，Skill 只需要 KV 能力 —— 拆开后 SQL backend 可以针对性优化
- **可组合**：可以 sessions 走 MySQL（多端共享），skills 走本地文件 —— 不同存储域对一致性/延迟/共享的要求不同

---

## 4. Repository Traits 定义

```rust
use async_trait::async_trait;

// ─── Session Repository ───
#[async_trait]
pub trait SessionRepo: Send + Sync {
    async fn load_all(&self) -> anyhow::Result<Vec<SessionMeta>>;
    async fn save_all(&self, sessions: &[SessionMeta]) -> anyhow::Result<()>;
}

// ─── Message Repository ───
#[async_trait]
pub trait MessageRepo: Send + Sync {
    async fn append(&self, session_id: &str, entry: &serde_json::Value) -> anyhow::Result<()>;
    async fn load(&self, session_id: &str, limit: usize) -> anyhow::Result<Vec<serde_json::Value>>;
    async fn save_all(&self, session_id: &str, records: &[serde_json::Value]) -> anyhow::Result<()>;
    async fn search(&self, query: &str, max_results: usize) -> anyhow::Result<Vec<SearchResult>>;
    async fn delete_session(&self, session_id: &str) -> anyhow::Result<()>;
}

// ─── API Cache Repository ───
#[async_trait]
pub trait ApiCacheRepo: Send + Sync {
    async fn save(&self, session_id: &str, messages: &[serde_json::Value]) -> anyhow::Result<()>;
    async fn load(&self, session_id: &str) -> anyhow::Result<Option<Vec<serde_json::Value>>>;
    async fn delete(&self, session_id: &str) -> anyhow::Result<()>;
}

// ─── Plan Steps Repository ───
#[async_trait]
pub trait PlanStepsRepo: Send + Sync {
    async fn save(&self, session_id: &str, steps: &[PlanStep]) -> anyhow::Result<()>;
    async fn load(&self, session_id: &str) -> anyhow::Result<Vec<PlanStep>>;
    async fn delete(&self, session_id: &str) -> anyhow::Result<()>;
}

// ─── Memory Repository ───
#[async_trait]
pub trait MemoryRepo: Send + Sync {
    async fn load(&self, agent_id: &str) -> anyhow::Result<CrossSessionMemory>;
    async fn save(&self, agent_id: &str, memory: &CrossSessionMemory) -> anyhow::Result<()>;
}

// ─── Stats Repository ───
#[async_trait]
pub trait StatsRepo: Send + Sync {
    async fn append(&self, record: &TokenRecord) -> anyhow::Result<()>;
    async fn append_batch(&self, records: &[TokenRecord]) -> anyhow::Result<()>;
    async fn read_range(&self, from: Option<i64>, to: Option<i64>) -> anyhow::Result<Vec<TokenRecord>>;
    async fn prune(&self, keep_days: u32) -> anyhow::Result<usize>;
}

// ─── Skill Repository ───
#[async_trait]
pub trait SkillRepo: Send + Sync {
    async fn list(&self, agent_id: &str) -> anyhow::Result<Vec<SkillEntry>>;
    async fn get(&self, agent_id: &str, name: &str) -> anyhow::Result<Option<SkillDefinition>>;
    async fn install(&self, agent_id: &str, name: &str, content: &str) -> anyhow::Result<()>;
    async fn remove(&self, agent_id: &str, name: &str) -> anyhow::Result<()>;
    async fn list_executable(&self, agent_id: &str) -> anyhow::Result<Vec<SkillDefinition>>;
    async fn format_skills(&self, agent_id: &str) -> anyhow::Result<String>;
}

// ─── Tool Doc Cache Repository ───
#[async_trait]
pub trait ToolCacheRepo: Send + Sync {
    async fn load(&self, agent_id: &str) -> anyhow::Result<HashMap<String, String>>;
    async fn save(&self, agent_id: &str, docs: &HashMap<String, String>) -> anyhow::Result<()>;
}
```

---

## 5. SQL Backend Schema 设计

```sql
CREATE TABLE sessions (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    agent_id    TEXT NOT NULL DEFAULT 'default',
    state       TEXT NOT NULL DEFAULT 'Active',
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    message_count INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE messages (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id  TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    type        TEXT NOT NULL,
    text        TEXT NOT NULL DEFAULT '',
    name        TEXT,
    args        TEXT,
    result      TEXT,
    reasoning   TEXT,
    extra       TEXT,
    created_at  INTEGER NOT NULL
);
CREATE INDEX idx_messages_session ON messages(session_id);

CREATE TABLE api_cache (
    session_id  TEXT PRIMARY KEY REFERENCES sessions(id) ON DELETE CASCADE,
    messages    TEXT NOT NULL
);

CREATE TABLE plan_steps (
    session_id  TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    step_order  INTEGER NOT NULL,
    description TEXT NOT NULL,
    done        INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (session_id, step_order)
);

CREATE TABLE memory (
    agent_id    TEXT PRIMARY KEY,
    data        TEXT NOT NULL
);

CREATE TABLE token_records (
    id              TEXT PRIMARY KEY,
    timestamp       INTEGER NOT NULL,
    agent_id        TEXT NOT NULL,
    model           TEXT NOT NULL,
    provider        TEXT NOT NULL,
    prompt_tokens   INTEGER NOT NULL,
    completion_tokens INTEGER NOT NULL,
    total_tokens    INTEGER NOT NULL,
    has_tool_calls  INTEGER NOT NULL,
    tool_call_count INTEGER NOT NULL,
    react_rounds    INTEGER NOT NULL,
    success         INTEGER NOT NULL,
    latency_ms      INTEGER NOT NULL,
    estimated_cost_usd REAL NOT NULL
);
CREATE INDEX idx_token_ts ON token_records(timestamp);

CREATE TABLE skills (
    agent_id    TEXT NOT NULL,
    name        TEXT NOT NULL,
    content     TEXT NOT NULL,
    parameters  TEXT,
    PRIMARY KEY (agent_id, name)
);

CREATE TABLE tool_cache (
    agent_id    TEXT NOT NULL,
    tool_name   TEXT NOT NULL,
    doc         TEXT NOT NULL,
    PRIMARY KEY (agent_id, tool_name)
);
```

### 关键设计决策

- **message 稀疏列**：type 不同时填不同列（user→text, tool_call→name+args+result），加一个 `extra TEXT` JSON 兜底
- **memory 整存整取**：`memory` 表存完整 JSON blob，CrossSessionMemory 本就是整读整写
- **skill 用 agent_id + name 联合主键**，完美映射文件路径
- **plan_steps 拆行存储**：每个 step 一行，支持按 step_order 排序

---

## 6. Backend 实现

### 6.1 FileBackend（第一阶段，零迁移成本）

```
crates/claw/src/storage/
├── mod.rs              # pub mod file; trait 定义 + StorageConfig
├── file.rs             # FileBackend + 所有 file 实现
├── sql/                # 后续阶段
└── mongo/              # 后续阶段
```

每个 File* 实现直接把现有 `std::fs` 代码搬过去，零行为变化。

### 6.2 SQL Backend（第二阶段）

统一用 `sqlx`（编译时 query check + async + 支持 3 种数据库）：

```toml
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "mysql", "postgres"], optional = true }
```

### 6.3 MongoDB Backend（第三阶段）

| 文件概念 | MongoDB |
|---------|---------|
| index.json | `sessions` collection (one doc per session meta) |
| {id}.jsonl | `messages` collection (one doc per message, `session_id` 索引) |
| memory.json | `memories` collection (one doc per agent, `agent_id` 唯一索引) |
| usage.jsonl | `token_records` collection (TS 索引) |
| *.md 文件 | `skills` collection (`agent_id` + `name` 复合索引) |
| hot_docs_cache.json | `tool_caches` collection |

---

## 7. 配置方式

```toml
# config.toml
[storage]
backend = "file"                # file | sqlite | mysql | postgres | mongodb

[storage.sqlite]
path = "~/.i-rs/claw/claw.db"

[storage.mysql]
url = "mysql://user:pass@localhost/claw"

[storage.postgres]
url = "postgres://user:pass@localhost/claw"

[storage.mongodb]
url = "mongodb://localhost:27017"
database = "claw"
```

---

## 8. 迁移策略

1. **第一阶段**：定义 traits + FileBackend（纯搬代码，零行为变化，`cargo test` 全绿）
2. **第二阶段**：实现 SQLite backend（`sqlx` + feature flag），FileBackend 仍为默认
3. **第三阶段**：实现 MySQL / PG backend（共用 90% 的 sqlx 代码）
4. **第四阶段**：实现 MongoDB backend
5. **数据迁移**：提供 `i-rs-claw migrate --from file --to sqlite` 命令

---

## 9. 异步化决策

所有 repo trait 方法返回 `Future`（全异步）。理由：

- `session.rs` 的调用方（`core/mod.rs`）本来就在 tokio runtime 内
- 消息追加是高频操作，不应该 block_on
- StatsManager 的 buffer flush 天然适合异步
- File backend 内部用 `spawn_blocking` 包装同步 fs 操作

---

## 10. 文件结构

```
crates/claw/src/storage/
├── mod.rs                # pub mod file; pub mod sql; pub mod mongo; trait 定义 + StorageConfig
├── file.rs               # FileBackend + 所有 file 实现
├── sql/
│   ├── mod.rs            # SqlBackend (sqlx)
│   ├── migrate.rs        # sqlx::migrate! 宏
│   └── sessions.rs etc.
└── mongo/
    ├── mod.rs            # MongoBackend
    └── sessions.rs etc.
```

---

## 11. 预估

| 维度 | 当前 | 目标 |
|------|------|------|
| 存储介质 | JSON/JSONL 文件 | 可插拔 (file/sqlite/mysql/pg/mongo) |
| I/O 模型 | 同步 fs | 异步 trait |
| 查询能力 | 全量扫描 | SQL 支持索引+FTS，Mongo 支持索引+聚合 |
| 事务 | 无 | sql backend 支持 |
| 迁移 | 手动 | sqlx migrate |
| 代码改动 | - | trait 抽象层 + FileBackend 搬代码 |

**预估代码量**：~1500 行新增（traits 定义 200 + FileBackend 搬 ~600 + SQL backend ~500 + Mongo ~200），改动 ~200 行（SessionManager / AgentRuntimeStore / AppCore 的依赖注入）。
