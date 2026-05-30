# i-rs-agent-api 架构设计

> 基于 i-rs-claw 引擎提取的智能体 API 服务 — **开源核心 + SaaS 服务 + 多端覆盖**

---

## 整体愿景

```
                    ┌──────────────────────────────────────────┐
                    │          i-rs 智能体平台 (开源)            │
                    │                                          │
                    │  ┌────────────┐  ┌──────────────────┐    │
                    │  │ i-rs-claw  │  │  i-rs-agent-api  │    │
                    │  │  TUI 客户端 │  │  多租户 API 服务  │    │
                    │  │  (开发者用)  │  │  (商业核心)       │    │
                    │  └────────────┘  └────────┬─────────┘    │
                    │                            │              │
                    │              ┌─────────────┴─────────┐   │
                    │              │    i-rs-engine         │   │
                    │              │    (核心引擎, 公共库)    │   │
                    │              └───────────────────────┘   │
                    └──────────────────────────────────────────┘

                                   ┌──── API ────┐
                                   │             │
              ┌────────────────────┼──┬──┬──┬─────┼────────────────┐
              │                    │  │  │  │     │                │
         ┌────▼───┐   ┌────▼───┐ ┌▼──▼──▼──▼──┐ ┌▼───────┐   ┌───▼────┐
         │ iOS    │   │ 微信    │ │  Web       │ │ CLI    │   │ 第三方  │
         │ App    │   │ 小程序   │ │ Dashboard  │ │ 工具    │   │ API    │
         └────────┘   └────────┘ └────────────┘ └────────┘   └────────┘
                          SaaS 用户 (对底层引擎不可见)
```

**分层逻辑**：

| 层 | 组件 | 用户可见性 | 说明 |
|---|------|-----------|------|
| **引擎** | `i-rs-engine` | 不可见 | 从 claw 提取的 ReAct 循环、LLM Provider、工具注册表、MCP |
| **API** | `i-rs-agent-api` | 开发可见 | 多租户 API 服务，开源可自部署 |
| **客户端** | App / 小程序 / Web / CLI | 用户可见 | 所有客户端通过同一 API 通信 |
| **TUI** | `i-rs-claw` | 开发者可见 | 独立 TUI 客户端，本地直连引擎（不入 API） |

---

## 商业模式

```
                    ┌──────────────────┐
                    │   cloud.i-rs.dev  │  ← SaaS 托管服务
                    │   (官方运营)      │
                    └────────┬─────────┘
                             │
              ┌──────────────┴──────────────┐
              │          用户群              │
              │                              │
   ┌──────────┴──┐   ┌─────────┴───┐   ┌────┴──────────┐
   │ Free Tier   │   │ Pro        │   │ Enterprise    │
   │             │   │            │   │               │
   │ 50 req/day  │   │ 无限请求   │   │ 自部署         │
   │ 基础模型    │   │ 高级模型   │   │ 私有云         │
   │ 基础工具集  │   │ 自定义工具 │   │ SLA 保障       │
   │ 社区支持    │   │ 优先支持   │   │ 专属部署       │
   └─────────────┘   └────────────┘   └───────────────┘
```

**开源策略**：
- 所有代码（engine + API + 客户端）在 GitHub 开源
- SaaS 服务 `cloud.i-rs.dev` 给不想自部署的用户
- 企业客户可以自部署 `i-rs-agent-api`
- 社区贡献者可以参与引擎/工具链的开发

---

## 架构总览

```
┌──────────────────────────────────────────────────┐
│                  i-rs-agent-api                   │
│                                                  │
│  ┌──────────┐  ┌──────────────────────────┐     │
│  │ Auth     │  │  API 路由 (Axum)          │     │
│  │ Layer    │  │  POST /v1/chat → engine  │     │
│  │          │  │  POST /v1/sessions       │     │
│  │ API Key  │  │  GET/PUT /v1/agents/     │     │
│  │ → tenant │  │  GET /v1/sessions/{id}   │     │
│  │ JWT      │  │  POST /v1/memory         │     │
│  │ → user   │  └──────────┬───────────────┘     │
│  └──────────┘             │                      │
│                    ┌──────▼─────────────────┐    │
│                    │    Engine Core          │    │
│                    │  (i-rs-engine crate)    │    │
│                    │                         │    │
│                    │  chat_loop (ReAct)      │    │
│                    │  ToolCallExecutor       │    │
│                    │  LlmProvider (多模型)    │    │
│                    │  ToolRegistry           │    │
│                    │  McpRegistry            │    │
│                    │  ContextManager         │    │
│                    │  SkillStore             │    │
│                    └──────┬──────────────────┘    │
│                           │                       │
│  ┌──────────┐  ┌─────────▼──────────┐  ┌───────┐ │
│  │PostgreSQL│  │   S3/对象存储       │  │ Redis │ │
│  │          │  │                    │  │       │ │
│  │ tenants  │  │ sessions/{t}/     │  │  rate  │ │
│  │ users    │  │   {u}/{id}.jsonl  │  │  limit │ │
│  │ api_keys │  │ memory/{t}/{u}    │  │ cache  │ │
│  │ billing  │  │ tool_cache/       │  │       │ │
│  └──────────┘  └────────────────────┘  └───────┘ │
└──────────────────────────────────────────────────┘
```

---

## Crate 结构

```
crates/
├── claw/                  # 保持现有 TUI，改依赖 engine
│   ├── src/
│   │   ├── core/          # AppCore、SessionManager（TUI 相关状态）
│   │   ├── ui/            # ratatui 渲染（不变）
│   │   ├── tui/           # 事件循环（不变）
│   │   └── app.rs         # TUI 应用状态
│   └── Cargo.toml         # dep: i-rs-engine
│
├── agent-engine/          # ← 新库，从 claw 提取的核心引擎
│   ├── src/
│   │   ├── providers/     # LlmProvider trait + OpenAI/Anthropic/Ollama
│   │   ├── llm.rs         # LlmEvent, StreamResult, ToolCallAcc
│   │   ├── tools/         # ToolRegistry, ClawTool, IrsTool, TOOL_INDEX
│   │   ├── mcp.rs         # McpRegistry, McpClient
│   │   ├── skill_store.rs # SkillDefinition, SkillStore
│   │   ├── tool_cache.rs  # ToolDocCache
│   │   ├── chat_loop.rs   # ReAct 循环（engine/mod.rs 重构）
│   │   ├── builder.rs     # 消息构建、系统提示词
│   │   ├── execution.rs   # execute_tool_call
│   │   ├── executor.rs    # ToolCallExecutor
│   │   ├── context.rs     # ContextManager (token 压缩)
│   │   └── utils.rs       # 工具函数
│   └── Cargo.toml         # 轻量依赖（无 ratatui/crossterm）
│
└── agent-api/             # ← 新服务，多租户 API
    ├── Cargo.toml         # dep: i-rs-engine, axum, sqlx, aws-sdk, redis
    └── src/
        ├── main.rs        # Axum 启动、中间件注册
        ├── auth.rs        # API Key / JWT 验证中间件
        ├── config.rs      # 多租户配置管理
        ├── routes/
        │   ├── mod.rs     # 路由注册
        │   ├── chat.rs    # POST /v1/chat/completions (SSE)
        │   ├── sessions.rs# CRUD /v1/sessions
        │   ├── agents.rs  # 查询 agent 配置
        │   └── memory.rs  # 读写用户记忆
        ├── storage/
        │   ├── mod.rs     # Storage trait 抽象
        │   ├── postgres.rs# Tenant/User/Key 存储
        │   ├── object.rs  # S3/MinIO 适配
        │   └── cache.rs   # Redis 适配
        └── billing.rs     # 用量统计 + 计费记录
```

---

## 客户端架构

所有客户端依赖同一 API，底层引擎对用户透明：

```
┌──────────────────────────────────────────────────────────────────┐
│                         API 层 (i-rs-agent-api)                    │
│                   POST /v1/chat/completions                       │
│                   认证 → 租户隔离 → 引擎调用                       │
└────────────┬────────────────────────────┬────────────────────────┘
             │                            │
             ▼                            ▼
┌──────────────────────┐    ┌──────────────────────────┐
│  终端用户客户端       │    │  开发者/高级用户           │
│                      │    │                          │
│  IrsClawApp (SwiftUI)│    │  i-rs-claw TUI            │
│  ├── macOS           │    │  ├── 直接调用 engine       │
│  ├── iPad            │    │  ├── 或可选连 API          │
│  └── iOS             │    │  ├── 本地工具执行           │
│                      │    │  └── 全量工具集             │
│  IrsClawMiniProgram  │    │                          │
│  ├── 微信小程序       │    │  CLI 工具链                │
│  └── 移动端查询/录入   │    │  ├── i-rs-* (70+)         │
│                      │    │  └── i-rs (统一入口)        │
│  Web Dashboard       │    │                          │
│  ├── 数据可视化       │    │  API Client SDK           │
│  └── AI 对话         │    │  ├── Python               │
│                      │    │  ├── JavaScript           │
│ 第三方集成            │    │  └── Rust                 │
│  ├── Cursor/VS Code  │    │                          │
│  └── Slack Bot       │    └──────────────────────────┘
└──────────────────────┘
```

**终端用户客户端**：面向普通用户，通过 API 使用智能体能力，完全不知道底层引擎。
**开发者/高级用户**：可以直接使用 i-rs-claw TUI（本地引擎）、CLI 工具链、API SDK。

---

## 用户模型

```
Tenant (租户/组织)
  ├── id: uuid
  ├── name: string
  ├── plan: "free" | "pro" | "enterprise"
  ├── config: AgentConfig      # 每个租户可配自己的 model/provider
  ├── api_keys: [ApiKey]       # 多个 key，可分别设置权限
  │
  ├── User (终端用户)
  │   ├── id: uuid            # API 调用时传 user="xxx"
  │   ├── tenant_id           # 所属租户
  │   ├── sessions: [Session]  # 只属于该用户
  │   ├── memory: CrossSessionMemory
  │   └── preferences
  │
  └── Billing
      ├── token_usage_monthly
      ├── rate_limit_per_second
      └── tool_quota_per_month
```

**API Key 层级控制**：

```
sk-{agent_id}-{random_secret}

sk-default-abc123          → 用 default agent
sk-analyst-def456          → 用 analyst agent
sk-admin-ghi789            → 管理权限（管理配置）
```

---

## API 合约

### 核心：OpenAI 兼容的 Chat Completions

```http
POST /v1/chat/completions
Authorization: Bearer sk-{tenant_key}
Content-Type: application/json

{
  "model": "analyst",            # → 映射 claw 的 agent_id
  "messages": [
    {"role": "user", "content": "分析支出趋势"}
  ],
  "tools": [{"type": "function", "function": {"name": "i-rs-ledger"}}],
  "stream": true,
  "user": "user_xxx"            # 可选，会话/记忆隔离
}
```

### 流式响应 (SSE)

```
data: {"type": "token", "content": "根据"}
data: {"type": "token", "content": "你的"}
data: {"type": "tool_call", "name": "i-rs-ledger", "arguments": "{\"subcommand\":\"expense\"}"}
data: {"type": "tool_result", "name": "i-rs-ledger", "content": "近30天支出汇总": ...}
data: {"type": "token", "content": "从数据来看"}
data: {"type": "usage", "prompt_tokens": 452, "completion_tokens": 128}
data: [DONE]
```

### 非流式响应

```json
{
  "id": "chatcmpl-xxx",
  "object": "chat.completion",
  "model": "analyst",
  "choices": [{
    "index": 0,
    "message": {
      "role": "assistant",
      "content": "根据你的支出数据..."
    }
  }],
  "usage": {
    "prompt_tokens": 452,
    "completion_tokens": 128
  }
}
```

### 其他接口

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/v1/sessions` | 创建新会话 |
| GET | `/v1/sessions` | 列会话列表 |
| GET | `/v1/sessions/{id}` | 获取会话消息 |
| DELETE | `/v1/sessions/{id}` | 删除会话 |
| GET | `/v1/agents` | 列出可用 agent |
| GET | `/v1/agents/{id}` | 获取 agent 详情 |
| PUT | `/v1/agents/{id}/config` | 更新 agent 配置 |
| GET | `/v1/memory` | 获取用户记忆 |
| POST | `/v1/memory` | 更新用户记忆 |
| GET | `/v1/usage` | 查询用量 |

---

## 存储设计

| 数据 | 存储 | Key 模式 |
|------|------|----------|
| 租户/用户/API Key | PostgreSQL | 关系表 |
| 会话消息 | S3/本地 | `{tenant}/{user}/{session_id}.jsonl` |
| Memory | S3/本地 | `{tenant}/{user}/memory.json` |
| 用量统计 | PostgreSQL | tenant + 时间分区 |
| 限流计数 | Redis | `ratelimit:{tenant}:{endpoint}` |
| Tool Cache | Redis/本地 | `toolcache:{tenant}:{tool_name}` |

存储层通过 trait 抽象，开发时用本地文件，部署时切换到云存储：

```rust
#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn list(&self, tenant: &str, user: &str) -> Result<Vec<SessionMeta>>;
    async fn get(&self, tenant: &str, user: &str, id: &str) -> Result<Session>;
    async fn append(&self, tenant: &str, user: &str, id: &str, msg: &Value) -> Result<()>;
    async fn delete(&self, tenant: &str, user: &str, id: &str) -> Result<()>;
}
```

---

## 引擎提取要点

### 从 claw 提取到 engine 的组件（无需修改）

| 组件 | 路径（claw） | 说明 |
|------|-------------|------|
| `LlmProvider` trait + 实现 | `providers/` | 完整提取，零修改 |
| `LlmEvent` | `llm.rs` | 完整提取 |
| `ToolRegistry` + `ClawTool` | `tools/mod.rs` | 完整提取 |
| `IrsTool` | `tools/i_rs.rs` | 完整提取 |
| `TOOL_INDEX` | `tools/index.rs` | 完整提取 |
| `McpRegistry` | `mcp.rs` | 完整提取 |
| `ToolCallExecutor` | `core/executor.rs` | 完整提取 |
| `ContextManager` | `core/context.rs` | 完整提取 |
| `SkillStore` | `skill_store.rs` | 完整提取 |
| `ToolDocCache` | `tool_cache.rs` | 完整提取 |
| `chat_loop` | `core/engine/mod.rs` | 提取，仅改 import |
| `execute_tool_call` | `core/engine/execution.rs` | 完整提取 |

### 需要重构

**`builder.rs`**：当前依赖 `crate::app::Message`（TUI 显示消息类型），改为用 `serde_json::Value`：

```rust
// 当前（claw）— 耦合 app::Message
pub struct MessageBuildParams<'a> {
    pub app_messages: &'a [crate::app::Message],
    // ...
}

// 重构后（engine）— 用 OpenAI 格式
pub struct MessageBuildParams<'a> {
    pub api_messages: &'a [Value],
    // ...
}
```

Impact：`AppCore::build_messages_for()`（留在 claw）负责 `app::Message → Value` 转换。

---

## 引擎调用示意

```rust
// agent-api 中的核心 handler
async fn chat_handler(
    tenant: TenantContext,
    body: ChatRequest,
    state: AppState,
) -> Response {
    let agent_id = &body.model;
    let user_id = body.user.as_deref().unwrap_or("default");

    let provider = create_provider(&tenant.config)?;
    let messages = body.messages;  // 已是 Value 格式
    let (tx, mut rx) = mpsc::unbounded_channel::<LlmEvent>();

    tokio::spawn(async move {
        chat_loop(provider, tenant.config, messages, tx, mcp_registry, skills).await;
    });

    Sse::new(rx.map(|event| match event {
        LlmEvent::Token(text) => Ok::<_, Infallible>(Event::data().data(text)),
        LlmEvent::ToolCall { name, args, .. } => Ok(Event::data().data(json!({...}))),
        LlmEvent::Done(_, usage) => Ok(Event::data().data(json!({...}))),
        // ...
    }))
}
```

---

## 可复用资源清单

| 项目 | 来源 | 状态 |
|------|------|------|
| 70+ CLI 工具包装 (i-rs-*) | crates/clis/ | 直接复用 |
| LLM Provider 实现 | claw/src/providers/ | 提取即用 |
| ReAct 循环 | claw/src/core/engine/ | 提取即用 |
| MCP 协议支持 | claw/src/mcp.rs | 提取即用 |
| 工具注册和执行 | claw/src/tools/ | 提取即用 |
| Axum 路由模式 | crates/cli-api/ | 参考模式 |
| 错误处理格式 | crates/cli-api/src/response.rs | 参考模式 |
| Token 压缩策略 | claw/src/core/context.rs | 提取即用 |

---

## 部署架构

```
                         ┌──────────────┐
                         │  Load        │
                         │  Balancer    │
                         └──────┬───────┘
                                │
                    ┌───────────┴───────────┐
                    │    agent-api:3000     │
                    │  (水平扩展 N 实例)      │
                    └───────────┬───────────┘
                                │
          ┌─────────────────────┼─────────────────────┐
          │                     │                     │
   ┌──────▼──────┐     ┌───────▼───────┐     ┌───────▼──────┐
   │ PostgreSQL  │     │  S3 / MinIO   │     │    Redis     │
   │ (tenants,   │     │  (sessions,   │     │  (rate limit,│
   │  users,     │     │   memory,     │     │   cache)     │
   │  billing)   │     │   cache)      │     │              │
   └─────────────┘     └───────────────┘     └──────────────┘
```

**无状态设计**：agent-api 实例不存本地状态，共享存储 → 水平扩缩容。

---

## 实施路线图

| 阶段 | 内容 | 估算 |
|------|------|------|
| **Phase 1** 引擎提取 | 创建 `agent-engine` crate，从 claw 提取 providers/tools/mcp/chat_loop/executor，重构 builder.rs | 2 天 |
| **Phase 2** 基础架构 | 创建 `agent-api` crate，Axum 搭建、auth 中间件、PostgreSQL schema、存储层 trait | 1.5 天 |
| **Phase 3** 核心 API | POST /v1/chat/completions (SSE)、会话 CRUD、agent 查询 | 1.5 天 |
| **Phase 4** 租户功能 | 多租户隔离验证、API Key 轮换、记忆管理、用量统计 | 1 天 |
| **Phase 5** 生产化 | Dockerfile、健康检查、限流、文档、CI | 0.5 天 |

**总计：约 6-7 天 MVP**（可并行推进 Phase 1 + 客户端开发）

---

## 待决策问题

- **计费模式**：按 token / 按请求 / 按月订阅？
- **模型策略**：统一采购转售，还是让用户自带 API Key？
- **起始客户端**：先做 Web Dashboard 还是 App？
- **数据合规**：消息保存策略（保存多久？是否可删除？）
- **免费额度策略**：Free Tier 多少量合适？
