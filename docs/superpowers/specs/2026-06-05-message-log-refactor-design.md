# 消息持久化重构：从 save_all 改为 append-only typed log

## 背景

`save_chat_result` → `api_msgs_to_jsonl` → `save_all_messages` 的链路有结构性缺陷：

1. **schema 漂移**：流式期间 `append_message` 写入完整数据（含 `step`/`total_steps`），结束时 `save_all_messages` DELETE+INSERT 用 `api_msgs_to_jsonl` 产出的**缺字段**版本覆盖。
2. **反序列化静默失败**：`Message::ToolCall { step: usize, total_steps: usize }` 没标 `#[serde(default)]`，缺字段时 `serde_json::from_value` 报错，`filter_map(.ok())` 把它过滤掉。客户端 reload 会话后看不到任何 tool_call。
3. **双写路径**：TUI 走 `Message → message_to_jsonl → save_all_messages`（完整），Dashboard 走 `Value → api_msgs_to_jsonl → save_all_messages`（有损）。两条路径 schema 不一致。

止血方案（已合入）：给 `step` / `total_steps` 加 `#[serde(default)]`。
根治方案（本文档）：删掉 `save_all_messages`，改 append-only typed log，三个写入路径（TUI / Dashboard / Gateway）统一走同一个 accumulator。

## 目标

1. **append-only**：存储层没有 `save_all` / `update`，只有 `append` + `read` + `delete_session`。
2. **domain / storage 分离**：`StoredMessage`（可序列化、带 schema_version）≠ `Message`（内存对象、可带派生状态）。
3. **单一累加器**：`MessageAccumulator` 消费 `LlmEvent`，输出 `Vec<Message>`。TUI / Dashboard / Gateway 共用。
4. **跨方言**：SQLite / MySQL / PG 共享 `MessageLog` trait，通过 `define_sql_stores!` 宏生成 impl。
5. **schema 版本化**：每条记录带 `schema_version`，未来加字段时按版本路由反序列化。
6. **零兼容包袱**：用户清空数据，新 schema 从空白起步。

## 非目标

- 不做历史数据迁移
- 不动 `api_cache`（它是 LLM 上下文恢复缓存，与用户可见消息日志正交）
- 不动 SSE 协议（客户端不需要改）

## 设计

### 存储格式

#### `StoredMessage`（持久化层）

```rust
pub struct StoredMessage {
    pub seq: u64,              // 会话内单调递增
    pub schema_version: u16,   // 目前 1
    pub timestamp: i64,        // Unix 秒
    pub kind: MessageKind,
}

#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MessageKind {
    User { text: String },
    Assistant {
        text: String,
        #[serde(default)]
        reasoning: String,
        #[serde(default)]
        token_usage: Option<TokenUsage>,
    },
    ToolCall {
        name: String,
        args: String,
        result: String,
        #[serde(default)]
        step: usize,
        #[serde(default)]
        total_steps: usize,
    },
    Evaluation { tool: String, valid: bool, issues: Vec<String> },
    Quality {
        score: Option<f64>,
        complete: bool,
        references_valid: u32,
        issues: Vec<String>,
    },
    Image { path: String, alt_text: String, width: u32, height: u32, format: String },
    Error { text: String },
}
```

#### JSONL 行布局（文件后端）

```json
{"seq":1,"schema_version":1,"timestamp":1700000000,"kind":"user","text":"hello"}
{"seq":2,"schema_version":1,"timestamp":1700000001,"kind":"tool_call","name":"water","args":"{}","result":"✓","step":0,"total_steps":1}
```

每行一个 JSON 对象，扁平的 `seq` / `schema_version` / `timestamp` 加上 `MessageKind` 任意序列列。

#### SQL 表布局

**SQLite**:
```sql
CREATE TABLE messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    seq INTEGER NOT NULL,
    schema_version INTEGER NOT NULL DEFAULT 1,
    timestamp INTEGER NOT NULL,
    kind TEXT NOT NULL,
    payload TEXT NOT NULL,
    UNIQUE(session_id, seq)
);
CREATE INDEX idx_messages_session_seq ON messages(session_id, seq);
```

**MySQL**:
```sql
CREATE TABLE messages (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    session_id VARCHAR(36) NOT NULL,
    seq BIGINT NOT NULL,
    schema_version INT NOT NULL DEFAULT 1,
    timestamp BIGINT NOT NULL,
    kind VARCHAR(32) NOT NULL,
    payload LONGTEXT NOT NULL,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE,
    UNIQUE KEY uk_session_seq (session_id, seq)
) ENGINE=InnoDB;
```

**PostgreSQL**:
```sql
CREATE TABLE messages (
    id BIGSERIAL PRIMARY KEY,
    session_id VARCHAR(36) NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    seq BIGINT NOT NULL,
    schema_version INT NOT NULL DEFAULT 1,
    timestamp BIGINT NOT NULL,
    kind VARCHAR(32) NOT NULL,
    payload JSONB NOT NULL,
    UNIQUE(session_id, seq)
);
CREATE INDEX idx_messages_session_seq ON messages(session_id, seq);
```

### 存储接口

```rust
#[async_trait]
pub trait MessageLog: Send + Sync {
    async fn append(&self, session_id: &str, messages: &[StoredMessage]) -> Result<()>;
    async fn read(&self, session_id: &str, after_seq: Option<u64>, limit: usize) -> Result<Vec<StoredMessage>>;
    async fn next_seq(&self, session_id: &str) -> Result<u64>;
    async fn delete_session(&self, session_id: &str) -> Result<()>;
    async fn search(&self, query: &str, max_results: usize) -> Result<Vec<SearchResult>>;
}
```

**没有 `save_all`**。这是结构上消除 bug 类的关键。

`append` 用 `sqlx::QueryBuilder::push_values` 做批量插入，单事务原子提交。三方言共用。

`search` 跨方言差异：
- SQLite / MySQL：`payload LIKE '%query%'`
- PG：`payload->>'text' ILIKE ... OR payload->>'name' ILIKE ... OR payload->>'result' ILIKE ...`

通过 `define_sql_stores!` 宏的 `$search_sql` 参数注入方言特定 SQL。

### 累加器（单一来源）

```rust
pub struct MessageAccumulator {
    messages: Vec<Message>,
    pending_assistant: Option<AssistantBuilder>,
}

impl MessageAccumulator {
    pub fn new() -> Self;
    pub fn from_history(loaded: Vec<Message>) -> Self;
    pub fn apply(&mut self, event: &LlmEvent) -> ApplyOutcome;
    pub fn into_messages(self) -> Vec<Message>;
    pub fn snapshot(&self) -> &[Message];   // 不消耗，用于 TUI 实时渲染
}

pub struct ApplyOutcome {
    pub streaming_preview: Option<StreamingPreview>,
    pub finalized: Vec<Message>,
}

pub struct StreamingPreview {
    pub text: String,
    pub reasoning: String,
}
```

事件处理（详细映射）：

| LlmEvent 变体 | 累加器动作 | 产出 |
|---|---|---|
| `Token(t)` | append `t` 到 `pending_assistant.text` | `streaming_preview` 更新 |
| `Reasoning(r)` | append `r` 到 `pending_assistant.reasoning` | `streaming_preview` 更新 |
| `ToolExecuted { name, args, result, step, total_steps }` | 先 `flush_assistant`，再 push `Message::ToolCall` | `finalized` 增 1-2 条 |
| `NewRound` | `flush_assistant` | `finalized` 增 0-1 条 |
| `Done(msgs, usage, _trace_id)` | `flush_assistant`，把 `usage` 回填到本轮所有 Assistant | `finalized` 增 0-1 条 |
| `Evaluation { .. }` | `flush_assistant`，push `Message::Evaluation` | `finalized` 增 1-2 条 |
| `ImageGenerated { .. }` | `flush_assistant`，push `Message::Image` | `finalized` 增 1-2 条 |
| `Error(e)` | `flush_assistant`，push `Message::Error` | `finalized` 增 1-2 条 |
| `Status(_)` / `PlanProgress(_)` / `HttpLog(_)` / `UsageRecord(_)` | 不持久化（运行时事件） | 空 |

`flush_assistant` 把 `pending_assistant` 转成 `Message::Assistant` 入 `messages`，并清空。

### 三条写入路径

**TUI (`tui/handlers/llm.rs`)**：

```rust
// 当前：app.messages 是 Vec<Message>，由各个 handler 维护
// 重构后：app.messages 由 accumulator 维护，handler 只 dispatch LlmEvent

let mut acc = MessageAccumulator::from_history(app.messages.drain(..).collect());
while let Some(event) = rx.recv().await {
    let outcome = acc.apply(&event);
    app.messages.extend(outcome.finalized);
    app.streaming_preview = outcome.streaming_preview;
}
// Done 之后
let final_msgs = acc.into_messages();
log.append(&sid, &final_msgs.iter().map(to_stored).collect())?;
```

**Dashboard SSE (`dashboard/routes.rs::chat_stream`)**：

```rust
let stream = unfold(
    (Some(rx), MessageAccumulator::from_history(loaded), state, sid),
    |(rx_opt, mut acc, state, sid)| async move {
        let mut rx = rx_opt?;
        let event = rx.recv().await?;
        let outcome = acc.apply(&event);
        // 把 streaming_preview 也通过 SSE 推给客户端（client 当前已经接 token/reasoning 事件）
        let sse = build_sse(&event);
        if matches!(event, LlmEvent::Done(..)) {
            let msgs = acc.into_messages();
            state.log.append(&sid, &msgs.iter().map(to_stored).collect()).await?;
            return Some((Ok(sse), (None, acc, state, sid)));
        }
        Some((Ok(sse), (Some(rx), acc, state, sid)))
    },
);
```

**Gateway (`gateway/mod.rs`)**：

```rust
let mut acc = MessageAccumulator::new();
while let Some(event) = rx.recv().await {
    acc.apply(&event);
}
let final_msgs = acc.into_messages();
let mut to_append = vec![StoredMessage::user(text, now())];
to_append.extend(final_msgs.iter().map(to_stored));
log.append(&sid, &to_append)?;
```

### 删除的代码

| 路径 | 删除 |
|---|---|
| `crates/claw/src/core/mod.rs` | `save_chat_result`, `api_msgs_to_jsonl`（~100 行） |
| `crates/claw/src/session.rs` | `save_all_messages`, `append_message`（~40 行，迁到 MessageLog） |
| `crates/claw/src/tui/clipboard.rs` | `save_session_messages`（~14 行） |
| `crates/claw/src/storage/mod.rs` | `MessageRepo` trait（含 `save_all` 方法） |
| `crates/claw/src/storage/sql/mod.rs` | macro 中的 `MessageRepo` impl 段 |
| `crates/claw/src/storage/file.rs` | `MessageRepo` impl 段 |

净删除约 250 行。新增 `crates/claw/src/message/{mod.rs,accumulator.rs}` 约 200 行。

### 保留的代码

- `api_cache` (`ApiCacheRepo`)：LLM 上下文恢复缓存，独立关注点。
- `Message` enum 名称与字段（领域类型沿用）。
- SSE 协议：客户端不需要改。
- `Message::ToolCall` 上的 `#[serde(default)]`：作为防御性兜底（万一磁盘上有未迁移的老数据）。

### Schema 版本化策略

`StoredMessage.schema_version = 1` 是基线。未来：

- **加字段**：新字段在 `MessageKind` 的变体里标 `#[serde(default)]`，老 v1 数据可正常读取（缺字段默认值），写入仍是 v1（或可升级到 v2 视情况）。
- **重命名字段**：bump v2，新增 `migrate_v1_to_v2` 函数。`StoredMessage::schema_version` 决定走哪个 reader。
- **重构枚举**：同 v2 路径。

### 关键不变量（测试锁定）

1. **append-only**：`MessageLog` trait 无 `save_all` / `update`。
2. **单调 seq**：`next_seq(sid) > max(read(sid, None, ∞).map(|m| m.seq))`。
3. **无损往返**：`Message → StoredMessage → Message` 等同。
4. **三路径一致**：相同 `LlmEvent` 流喂给 accumulator，TUI / Dashboard / Gateway 输出相同的 `Vec<Message>`（**golden test**）。
5. **回归锁定**：流 `ToolExecuted + Done` 后 append → reload → tool_call 存在且 step/total_steps 完整。

### 模块布局

```
crates/claw/src/
├── message/                  # NEW
│   ├── mod.rs                # StoredMessage, MessageKind, Message (domain)
│   └── accumulator.rs        # MessageAccumulator (LlmEvent → Vec<Message>)
├── storage/
│   ├── mod.rs                # MessageLog trait（替代 MessageRepo）
│   ├── file.rs               # JSONL 实现
│   └── sql/
│       ├── mod.rs            # define_sql_stores! macro（扩展 MessageLog 段）
│       ├── sqlite.rs         # DDL + macro 调用
│       ├── mysql.rs          # DDL + macro 调用
│       └── postgres.rs       # DDL + macro 调用
├── session.rs                # 精简：不再负责持久化
├── tui/
│   ├── handlers/llm.rs       # 用 accumulator 替代直接 mutate app.messages
│   ├── handlers/key.rs       # 切换会话用 MessageLog.read 重建
│   └── clipboard.rs          # 删除 save_session_messages
├── dashboard/routes.rs       # chat_stream 和 send_message 用 accumulator
├── gateway/mod.rs            # 用 accumulator
└── core/mod.rs               # 删除 save_chat_result / api_msgs_to_jsonl
```

### 实施顺序

1. 新增 `crates/claw/src/message/`（types + accumulator，无 caller）。
2. 扩展 `MessageLog` trait + File 实现，与 `MessageRepo` 并存。
3. 扩展 `define_sql_stores!` 宏生成 `MessageLog` impl；三方言各自 DDL。
4. **Golden test**：accumulator 行为锁定。
5. **Storage round-trip test**：SQLite in-memory 全路径。
6. 迁移 Dashboard 路径（routes.rs 两处）。
7. 迁移 TUI 路径（4 处 save_session_messages）。
8. 迁移 Gateway 路径。
9. 删除 `MessageRepo` trait、`save_chat_result`、`api_msgs_to_jsonl`、`save_all_messages`、`session.rs::append_message`。
10. 清用户数据。

### 风险与缓解

- **风险**：三 caller 迁移顺序敏感。**缓解**：新存储与旧并存到所有 caller 都迁完。
- **风险**：accumulator 有 bug 导致 TUI / Dashboard 不一致。**缓解**：步骤 4 的 golden test。
- **风险**：seq 边界条件。**缓解**：对 `read(after_seq=N)` 加 property test。

### 用户数据清理

用户主动清空 `~/.i-rs/claw/claw.db`（或对应的 sql_url 数据库）+ `~/.i-rs/claw/sessions/*.jsonl` 后启动新版本。新 `migrate()` 函数建表，无升级路径。

## 验收

- `cargo check --workspace` 0 errors 0 warnings
- `cargo test -p i-rs-claw -- --test-threads=1` 全绿（包含新增 5+ 个测试）
- 小程序 / web / TUI 三端 reload 会话后 tool_call 消息正确显示，含 step/total_steps
