# i-rs-claw 进化史：从终端助手到 AI 操作系统

## 缘起

2024 年初，i-rs 项目已经有了 70 个 CLI 工具——覆盖个人数据管理的方方面面。但有个问题越来越明显：**用户记不住命令**。

- "我想查上周的体重趋势……是 `i-rs weight chart` 还是 `i-rs weight stats`？"
- "帮我看看冰箱里的牛奶过期没？"——这个需求需要组合查询 `i-rs best-by list` 和 `i-rs grocery list`
- "我昨天跑了 5 公里，帮我记上。"——需要先找到正确的工具，再构造参数

70 个 CLI 工具虽强大，但心智负担高。我们需要一个**自然语言接口**。

## v0.1: 第一个 TUI 原型

最初的 claw 非常简单：

```
i-rs claw chat
```

一个 ratatui 终端界面，支持 OpenAI 兼容 API 的流式对话。用户可以输入自然语言，LLM 调用 i-rs CLI 工具执行操作。

**关键决策**（至今未变）：
- **不依赖 i-rs-core**——claw 通过 `std::process::Command` 调 CLI 二进制，而非直接调用 Rust API
- **文件存储**——会话和消息存在 `~/.i-rs/claw/` 目录的 JSONL 文件中
- **ReAct 循环**——LLM 流式返回 → 检测 tool_calls → 执行 → 结果塞回 → 继续请求 → 直到 LLM 返回文本

## v0.2: 多 Agent + 多模型

一个月后，一个问题浮现：**一个模型不够用**。

- GPT-4o 擅长日常对话，但分析数据不如 o3-mini
- 不同任务需要不同 prompt（系统提示词）
- 用户想在某些场景用本地模型（Ollama），省 API 费用

解决方案是 **Agent 系统**：

```toml
# config.toml
provider = "openai"
model = "gpt-4o-mini"

[agents.chatgpt]
model = "gpt-4o"

[agents.claude]
provider = "anthropic"
api_key = "sk-ant-xxx"

[sub_agents.analyst]
model = "o3-mini"
capabilities = ["数据分析", "代码生成"]
```

每个 Agent 拥有独立的：
- Provider / Model 配置
- 系统提示词（`prompts/` 目录）
- 工具集（可启用/禁用特定工具）
- 会话历史（独立上下文管理）

新增的 **TaskRouter** 负责将子任务路由到合适的 Sub-agent。比如用户说 "分析我的跑步数据并画个图表"，Router 会：
1. 主 Agent 调用 `delegate` 工具
2. 将数据分析任务派给 `analyst` sub-agent
3. 将结果返回给主 Agent
4. 主 Agent 调用 `chart` 工具生成图表

## v0.3: 工具系统大爆发

Agent 有了，但工具只有 `i_rs` 一个。为了让 claw 真正有用，我们开始疯狂加工具：

| 版本 | 新增工具 | 说明 |
|------|---------|------|
| v0.2 | `i_rs` | 调用 70 个 CLI 工具 |
| v0.3 | `web_search` | 联网搜索（内置搜索引擎） |
| v0.3 | `calculator` | 精确计算（避免 LLM 算错数） |
| v0.3 | `chart` | 数据可视化（生成 SVG 图表） |
| v0.4 | `file_ops` | 文件读写操作 |
| v0.4 | `search_conversations` | 历史会话全文搜索 |
| v0.5 | `user_memory` | 跨会话用户记忆（长期记忆） |
| v0.5 | `delegate` | 子 Agent 任务委托 |
| v0.6 | `vision_tool` | 图片分析 |
| v0.6 | `skill_tool` | AI 技能动态注册 |

工具系统的架构也演化清晰：

```rust
#[async_trait]
pub trait ClawTool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameter_schema(&self, enabled_cli_tools: &[&str]) -> Value;
    async fn execute(&self, args: &Value, ctx: &ToolContext) -> Result<String, ClawError>;
}
```

`ToolRegistry` 统一管理所有工具，支持：
- 内置工具（编译时确定）
- Skill 工具（运行时从 SkillStore 动态注册）
- MCP 工具（通过 MCP 协议自动发现）
- i-rs CLI 工具（自动扫描 crate 列表）

## v0.5: 长期记忆系统

"我上周跟你说过我养了一只猫，你还记得吗？"——这要求 claw **跨会话记住用户信息**。

我们实现了三级记忆架构：

```
第一级: 上下文记忆 (Session Context)
  ├── 当前会话的所有消息
  ├── Token 超限时自动压缩 (smart_compress)
  └── 策略: 删除中间轮次, 保留首尾

第二级: 跨会话记忆 (CrossSessionMemory)
  ├── 关键事实存储 (key-value)
  ├── 用户偏好 / 个人信息
  └── 自动提取 + 显式记忆

第三级: 语义搜索 (SemanticSearch)
  ├── TF-IDF + 向量嵌入
  ├── 召回相关历史会话
  └── 作为上下文注入
```

记忆系统让 claw 从一个"无状态助手"变成了"了解你的私人助理"。

## v0.7: MCP 协议集成

2024 年中，Anthropic 发布了 MCP (Model Context Protocol) 协议。我们迅速跟进：

```
i-rs claw chat              ← TUI 直接对话
    ↓
AppCore.chat_loop()         ← ReAct 主循环
    ↓
ToolRegistry                ← 内置 + MCP 工具统一调度
    ↓
McpRegistry                 ← MCP 协议客户端
    ↓
MCP Server (stdio)          ← 外部工具服务器
    ↓
Playwright / Filesystem / ...  ← 任意 MCP 兼容服务
```

用户可以在 `config.toml` 中配置任意 MCP 服务器：

```toml
[[mcp_servers]]
name = "playwright"
transport_type = "stdio"
command = "npx @anthropic-ai/claude-code-mcp"
```

## v0.8: Gateway — 社交平台接入

终端虽好，但用户不可能随时开着终端。于是我们加了 Gateway 层：

```
Gateway 架构:

Telegram Bot  ←→  i-rs-claw gateway  ←→  LLM
WeChat 消息    ↕                       ↕
(更多平台)     AppCore (复用同一套逻辑)  ToolRegistry
```

Gateway 重用了 TUI 模式下的全部代码（`AppCore`、`ToolRegistry`、`SessionManager`），只是在输入/输出通道上做了抽象。同一个 AI 大脑，既能在终端用，也能在 Telegram/微信里用。

## v0.9: Dashboard — 可观测性

"LLM 到底在想什么？为什么调用这个工具？花了多少 token？"

为了解决黑盒问题，我们构建了 Web Dashboard：

```
i-rs claw dashboard
  → 内嵌 Axum 服务器 (http://0.0.0.0:3000)
  → React SPA (dashboard-ui 子目录, rust-embed 编译进二进制)
  → API: 会话列表 / 消息详情 / Token 统计 / 工具调用追踪
  → CORS 保护 + Bearer Token 认证
```

Dashboard 使用 `--features dashboard` 编译，不增加不必要的依赖。

## v1.0: 可插拔存储层

最初的 JSONL 文件存储在高并发和多会话场景下开始暴露出问题：

- 写放大（每次追加都要读整个文件）
- 不支持并发写入
- 跨会话搜索需要全量扫描

于是我们进行了最大的一次重构：**可插拔存储层抽象**。

```rust
pub enum StorageBackend {
    File,      // 默认 (JSONL 文件)
    Sqlite,    // `--features sqlite`
    Mysql,     // `--features mysql`
    Postgres,  // `--features postgres`
}
```

每个存储领域都有独立的 Repository trait：

```rust
#[async_trait]
trait SessionRepo: Send + Sync {
    async fn list_sessions(&self) -> Result<Vec<SessionSummary>>;
    async fn load_session(&self, id: &str) -> Result<Option<Session>>;
    async fn save_session(&self, session: &Session) -> Result<()>;
    async fn delete_session(&self, id: &str) -> Result<()>;
}

#[async_trait]
trait MessageRepo: Send + Sync {
    async fn list_messages(&self, session_id: &str) -> Result<Vec<Message>>;
    async fn append_message(&self, session_id: &str, msg: &Message) -> Result<()>;
    // ...
}
```

`ClawStorage` 容器持有所有 Repository trait object，根据不同 feature 编译不同的后端实现：

```rust
pub struct ClawStorage {
    pub session: Arc<dyn SessionRepo>,
    pub message: Arc<dyn MessageRepo>,
    pub api_cache: Arc<dyn ApiCacheRepo>,
    pub plan_steps: Arc<dyn PlanStepRepo>,
    pub memory: Arc<dyn MemoryRepo>,
    pub token_records: Arc<dyn TokenRecordRepo>,
    pub skills: Arc<dyn SkillRepo>,
    pub tool_cache: Arc<dyn ToolCacheRepo>,
}
```

核心设计原则：
- **运行时选择**——存储后端在 config.toml 中配置，非编译时决定
- **cfg 守卫在 match 分支级别**——所有分支都通过类型检查，未启用的后端给出清晰错误
- **SQLite 是默认 SQL 后端**——零配置，文件存储在 `~/.i-rs/claw/claw.db`
- **迁移友好**——SQLite 使用 sqlx 内联迁移（`migrate!`），自动建表

8 张表的 SQLite 数据库（约 78 KB）替换了原先散落的 JSONL 文件：

```
sessions      → 会话元数据
messages      → 消息内容 (type, role, JSON body)
api_cache     → LLM 响应缓存
plan_steps    → Plan-then-Execute 中间步骤
memory        → 跨会话记忆
token_records → Token 用量统计
skills        → AI 技能定义
tool_cache    → 工具文档缓存
```

## 架构全景 (v1.0)

```
┌─────────────────────────────────────────────────────────┐
│                    用户交互层                            │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────┐  │
│  │ TUI      │  │ Telegram │  │ 微信     │  │ API    │  │
│  │ (ratatui)│  │ Gateway  │  │ Gateway  │  │ (HTTP) │  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └───┬────┘  │
│       │             │             │             │        │
├───────┴─────────────┴─────────────┴─────────────┴────────┤
│                   核心处理层 (AppCore)                     │
│  ┌──────────────────────────────────────────────────┐   │
│  │             ReAct Chat Loop                      │   │
│  │  stream → tool_calls → execute → loop → done     │   │
│  └────┬─────────────┬──────────────┬────────────────┘   │
│       │             │              │                     │
│  ┌────▼───┐  ┌──────▼──────┐  ┌───▼────────────┐       │
│  │ Context│  │ ToolExecutor│  │ Orchestrator   │       │
│  │Manager │  │ (timeout/   │  │ (Plan-then-    │       │
│  │(压缩)  │  │  retry/     │  │  Execute, 可选)│       │
│  │        │  │  parallel)  │  │                │       │
│  └────────┘  └──────┬──────┘  └────────────────┘       │
│                     │                                    │
├─────────────────────┴───────────────────────────────────┤
│                   工具 & 技能层                          │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌───────────┐  │
│  │ Tool     │ │ MCP      │ │ Skill    │ │ Agent     │  │
│  │ Registry │ │ Registry │ │ Store    │ │ Runtime   │  │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └─────┬─────┘  │
│       │            │            │               │        │
├───────┴────────────┴────────────┴───────────────┴────────┤
│                   存储层 (可插拔)                          │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌─────────┐ │
│  │ File     │  │ SQLite   │  │ MySQL    │  │PostgreSQL│ │
│  │ (JSONL)  │  │ (默认SQL) │  │ (可配置)  │  │ (可配置) │ │
│  └──────────┘  └──────────┘  └──────────┘  └─────────┘ │
└─────────────────────────────────────────────────────────┘
```

## 从 0 到 70 个工具，从 1 个文件到 253 个测试

回顾 claw 的进化历程，几个关键数字：

| 指标 | v0.1 | v1.0 |
|------|------|------|
| 源文件数 | ~15 | 67 |
| 内置工具 | 1 | 10 |
| 存储后端 | 1 (File) | 4 (File/SQLite/MySQL/PG) |
| 测试数 | 0 | 253 |
| Provider | 1 (OpenAI) | 4 (OpenAI/Anthropic/Ollama/Zhipu) |
| 接入平台 | 1 (TUI) | 4 (TUI/Telegram/微信/Dashboard) |
| Agent 支持 | 单 Agent | 多 Agent + Sub-agent |

## 架构决策回顾

### 做对的决策

1. **不依赖 i-rs-core** — 通过 CLI 二进制调用工具，保持松耦合。即使 i-rs-core 大改，claw 不受影响。

2. **ReAct 循环而非 Agent 框架** — 不引入 LangChain 之类的外部框架。ReAct 足够简单，完全可控。

3. **文件存储起步，SQL 跟进** — 一开始用文件存储快速迭代，当性能瓶颈出现时才引入 SQL 抽象。没有过度设计。

4. **cfg feature 守卫而非条件编译整个文件** — 在 match 分支级别加 `#[cfg]`，所有代码都通过类型检查，未启用的后端给出友好错误。既安全又易用。

5. **Dashboard 作为可选 feature** — 不增加不必要的编译时间和二进制体积。需要时 `--features dashboard` 即可。

### 值得改进的

1. **Plugin 系统仍在早期** — 动态加载外部插件需要更多打磨
2. **Gateway 线程模型** — Telegram/微信的长轮询 + ReAct 循环的并发控制可以更优雅
3. **Plan-then-Execute 模式** — 目前还是实验性功能，需要更多验证

## 未来方向

- **多模态输入** — 语音、图片、视频作为输入
- **更智能的上下文管理** — 基于语义而不是 Token 计数的压缩策略
- **插件市场** — 社区贡献的插件可以在 config 中一键启用
- **离线模式** — 本地模型 (Ollama) + 本地知识库，完全离线运行
- **iOS/原生 App** — IrsClawApp 正在开发中，复用 API 层
