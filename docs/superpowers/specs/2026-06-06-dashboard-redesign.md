# Dashboard 重新设计 Spec

**日期**: 2026-06-06 | **状态**: Draft

## 1. 核心定位

Dashboard 是 i-rs 生态系统**面向普通用户的主产品入口**。

当前问题：Dashboard 是 claw TUI 的内嵌 sidecar（rust-embed + HTTP server + React SPA），架构上 TUI 是主、Dashboard 是从。实际上 Dashboard 才是主力客户端——Web、小程序、App 三端共用同一个 API 出口。TUI 只是面向极少数终端用户的调试工具。

**翻转后的定位：**

| 组件 | 角色 | 用户 |
|------|------|------|
| **claw serve** | 主产品进程：API 网关 + Web Dashboard | 所有普通用户 |
| **claw tui** | 子命令：终端调试/专业模式（直连 core） | 开发者/终端爱好者 |
| **claw-core** | 纯 lib：AI 引擎（无 main.rs） | 被 serve 和 tui 共用 |
| **Web UI** | React SPA：Chat + Data + Agents + Usage + Settings | Web 端用户 |
| **小程序/App** | 独立客户端：通过 HTTP API 连接 serve | 移动端用户 |

## 2. 架构总览

### 2.1 进程拓扑

```
                    ┌──────────────────────────────────────────────┐
                    │            claw serve（主产品进程）             │
                    │                                              │
  浏览器/小程序/App ──→  HTTP API  ──→  claw-core（AI 引擎）        │
                    │    │                    │                     │
                    │    │             ┌──────┴──────┐              │
                    │    │          LLM  Tools  Memory              │
                    │    │             └──────┬──────┘              │
                    │    │                    │                     │
                    │    │       ┌────────────┴───────────┐         │
                    │    │       │  Storage Backend       │         │
                    │    └──────→│  PG/MySQL/Mongo/       │         │
                    │            │  SQLite/Redis          │         │
                    │            └────────────────────────┘         │
                    │  Web UI (SPA)  ← 构建产物嵌入 serve 静态服务   │
                    └──────────────────────────────────────────────┘

                    ┌──────────────────────┐
                    │  claw tui（子命令）   │
                    │  claw-core 直连       │
                    │  不走 HTTP API        │
                    └──────────────────────┘
```

### 2.2 Crate 结构变更

**现状：**
```
crates/claw/              ← TUI + Dashboard 混在一起
  src/main.rs             ← 主入口
  src/dashboard/          ← HTTP 服务器
  dashboard-ui/           ← React SPA（嵌入）
```

**目标：**
```
crates/claw-core/         ← [新] AI 引擎 lib（无 main.rs）
  src/lib.rs
  src/app.rs              ← AppCore（迁入）
  src/engine/             ← chat_loop, builder, execution（迁入）
  src/providers/          ← LLM providers（迁入）
  src/tools/              ← ToolRegistry（迁入）
  src/session.rs          ← SessionManager（迁入）
  src/memory.rs           ← CrossSessionMemory（迁入）
  src/config.rs           ← Config（迁入）
  src/stats/              ← 用量统计（迁入）
  src/storage/            ← 存储层（迁入）
  src/mcp.rs              ← MCP 客户端（迁入）
  src/semantic.rs          ← 语义搜索（迁入）

crates/claw/              ← 二进制 crate（serve + tui，同一二进制两个子命令）
  src/main.rs             ← CLI 子命令分发：`claw serve` / `claw tui`
  src/serve/              ← [新] serve 模式：API 路由 + 静态文件服务
  src/tui/                ← [迁入] 现有 TUI 逻辑（功能不做任何改动）
  assets/                 ← Web UI 构建产物（dashboard-ui/dist/，rust-embed 嵌入）
  dashboard-ui/           ← React SPA 工程（独立 pnpm dev，构建产出自動拷贝到 assets/）

crates/i-rs-api/          ← [现有] 70 个 CLI 工具的 REST 封装（保留不变）

  ※ claw 仍是单一二进制。`claw serve` 启动 HTTP 服务器，`claw tui` 启动终端界面。
  ※ TUI 直连 claw-core（零延迟），Web/小程序/App 通过 HTTP API。
```

### 2.3 数据流

```
用户发消息（Web/MiniProgram/App）
    │
    ▼
POST /api/chat { message, agent_id?, session_id? }
    │
    ▼
handler:
  1. 获取或创建 session
  2. persist 用户消息到 DB（通过 SessionManager）
  3. spawn chat_loop(tx) 到 tokio task
  4. 返回 SSE stream，subscribe tx 事件
    │
    ▼
chat_loop 执行 ReAct 循环（每次 LLM 返回时）：
    ├─ Token → emit SSE token，不 persist
    ├─ ToolExecuted → emit SSE + persist tool_call 到 DB
    ├─ NewRound → emit SSE + persist assistant 到 DB
    └─ Done → persist quality + emit SSE done
    ▼
SSE stream → 浏览器实时渲染

                           聊天期间的任何时刻：
                           DB 中已有当前轮次之前的所有消息
                           刷新/断连 → GET /api/sessions/{id} 即可恢复
```

**关键变更**：
1. chat_loop 不再是全量 accumulate 到 Done 才 persist，而是**每轮增量 persist**
2. POST /api/chat 直接返回 SSE，不再分两步
3. 所有客户端（Web/小程序/App）共享同一套 API 逻辑

### 2.4 与 i-rs-api 的关系

Dashboard 的 `/api/data/{tool}` 端点有两种实现选择：

| 方案 | 做法 | 适用 |
|------|------|------|
| **直接调用 service 层** | claw-core 依赖 i-rs-{name} crate，直接调 service 函数 | 单体部署，零网络开销 |
| **代理到 i-rs-api** | Dashboard 作为反向代理，透传请求到 i-rs-api 进程 | 微服务架构，解耦 |

**决策**：先用直接调用（方案 A），因为当前所有 CLI crate 已在 workspace 中。后续如需独立扩展 i-rs-api，再加代理模式。

## 3. API 设计

### 3.1 端点列表

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/chat` | 发消息，返回 SSE 流 |
| GET | `/api/chat/stream/{session_id}` | 重连 SSE 流 |
| GET | `/api/sessions` | 会话列表 |
| POST | `/api/sessions` | 新建会话 |
| GET | `/api/sessions/{id}` | 会话详情 + 消息历史 |
| DELETE | `/api/sessions/{id}` | 删除会话 |
| POST | `/api/sessions/{id}/feedback` | 消息反馈 |
| GET | `/api/agents` | Agent 列表 |
| POST | `/api/agents` | 创建 Agent |
| PUT | `/api/agents/{id}` | 更新 Agent |
| DELETE | `/api/agents/{id}` | 删除 Agent |
| GET | `/api/stats` | Token 用量统计 |
| GET | `/api/tools` | 工具列表 |
| GET | `/api/skills` | 技能列表 |
| GET | `/api/plugins` | 插件列表 |
| GET | `/api/config` | 配置（脱敏） |
| PUT | `/api/config` | 更新配置 |
| GET/POST/DELETE/PATCH | `/api/data/{tool}[/{id}]` | i-rs 数据工具代理 |

### 3.2 聊天 API 详细设计

**POST /api/chat**

Request:
```json
{
  "message": "string (required)",
  "agent_id": "string (optional, default: \"default\")",
  "session_id": "string (optional, 复用已有会话)"
}
```

Response: SSE stream
```
event: token
data: 这是

event: token
data: 一个

event: token
data: 回复

event: tool_executed
data: {"name":"i_rs","args":"{}","result":"...","step":0,"total_steps":3}

event: new_round
data: 

event: done
data: {"usage":{"prompt_tokens":100,"completion_tokens":50,"total_tokens":150},"quality":{"score":0.95,"complete":true}}
```

**GET /api/chat/stream/{session_id}?cursor={last_event_seq}**

断线重连。`cursor` 是前端记录的最后收到的 event 序号（整数，从 0 开始）。
服务端逻辑：
1. 如果正在 chat_loop 中 → 从 cursor 之后的事件开始继续 SSE
2. 如果 chat_loop 已结束 → 返回 done 事件，前端知道流已结束
3. cursor 在 done 事件中也返回，格式：`{"usage":...,"quality":...,"cursor":42}`

前端重连伪代码：
```typescript
let lastCursor = -1
function connect(sessionId: string) {
  const es = new EventSource(`/api/chat/stream/${sessionId}?cursor=${lastCursor + 1}`)
  es.onmessage = (e) => {
    lastCursor = e.lastEventId ? parseInt(e.lastEventId) : lastCursor
    handleEvent(e)
  }
  es.onerror = () => {
    es.close()
    setTimeout(() => connect(sessionId), 1000) // 自动重连
  }
}
```

## 4. 增量持久化

### 4.1 问题

当前 dashboard 后端用 MessageAccumulator 在内存中积累整个对话的所有事件，仅在 LlmEvent::Done 或 Error 时才调用 persist_messages。SSE 断开 → 所有消息丢失。

### 4.2 方案

chat_loop 每次返回一轮 Assistant 响应时（即 LlmEvent::NewRound），立即将本轮消息 persist 到 DB。同时在 LlmEvent::Done 时 persist 最终状态（包括 quality）。

```
chat_loop 伪代码：

for round in reAct_loop:
    emit SSE tokens
    if tool_calls:
        execute tools
        persist tool_call messages to DB  ← 新
        emit SSE tool_executed
    else:
        persist assistant response to DB   ← 新
        emit SSE new_round

persist final state + quality to DB
emit SSE done
```

## 5. 前端重新设计

### 5.1 页面结构

| 页面 | 路由 | 说明 |
|------|------|------|
| Chat | `/` | AI 对话（核心页面） |
| Data | `/data` | 70 个 i-rs 数据工具看板 |
| Sessions | `/sessions` | 会话历史管理 |
| Agents | `/agents` | Agent 配置管理 |
| Usage | `/usage` | Token 用量 + 费用统计 |
| Settings | `/settings` | 系统配置（Provider、模型等） |

### 5.2 技术决策

| 决策项 | 选择 | 原因 |
|--------|------|------|
| 路由 | wouter（~2KB）或保持手写 | 轻量，避免引入 react-router |
| 状态管理 | React Context | 不需要 Redux/Zustand，全局状态简单 |
| 样式 | CSS Modules | 组件级隔离，避免 1968 行单文件 |
| SSE 管理 | useChatStream hook | 封装连接/重连/事件分发 |
| Markdown | react-markdown + remark-gfm | 现有依赖不变 |
| Build | Vite | 现有不变 |

### 5.3 组件树

```
App
├── AuthGate（token 认证）
├── Sidebar
│   ├── Logo
│   ├── AgentSwitcher
│   ├── NavItems
│   └── ThemeToggle
├── ChatPage
│   ├── PageHeader（标题 + Agent badge + New Chat）
│   ├── MessageList
│   │   ├── MessageBubble（user/assistant/error/image）
│   │   │   ├── ReasoningBlock
│   │   │   ├── MarkdownRenderer
│   │   │   ├── ToolCallCard[]
│   │   │   ├── TokenUsageFooter
│   │   │   ├── QualityIndicator
│   │   │   └── FeedbackButtons
│   │   └── StreamingBubble
│   ├── ChatInput
│   └── SuggestChips
└── OtherPages...
```

## 6. 分布式部署

### 6.1 拓扑

```
         ┌─── Nginx / Cloud LB ───┐
         │   sticky: cookie       │
         │                        │
  ┌──────┴──────┐        ┌───────┴──────┐
  │ claw serve  │        │ claw serve   │
  │  instance-1 │        │  instance-2  │
  └──────┬──────┘        └───────┬──────┘
         │                       │
         └───────────┬───────────┘
                     │
              ┌──────┴──────┐
              │  PostgreSQL  │
              │  / MongoDB   │
              └─────────────┘
```

### 6.2 关键约束

- Sticky session 保证同一用户的 POST `/api/chat` 和 SSE `chat/stream` 落在同一实例
- API 层无本地状态，所有状态在 DB
- 实例宕机：LB 重路由到其他实例，客户端重连 SSE
- 静态资源（Web UI）可选择用 Nginx 直接 serve 或 CDN

## 7. 实施阶段

### Phase 1: 基础架构
1. 拆分 `claw-core` crate（lib only）
2. 将现有 claw 代码迁入 claw-core（保持功能不变）
3. 验证 `cargo check --workspace` 0 errors

### Phase 2: Serve 模式
1. 新建 `crates/claw/src/serve/` 模块
2. 从 `dashboard/` 迁入 route handlers
3. 实现增量持久化逻辑
4. 重构 POST /api/chat 为单端点 SSE
5. 移除旧的 `MessageAccumulator` 依赖（或用新方式）

### Phase 3: 前端重构
1. 添加路由（wouter）
2. 拆分 Chat.tsx → hooks + 组件
3. CSS Modules 化
4. 新增 Data 页面（i-rs 数据看板）
5. Streaming 重连逻辑

### Phase 4: 联调 & 测试
1. Web UI + API 联调
2. TUI 模式验证（确保不受影响）
3. 分布式部署测试（多实例 + PostgreSQL）

## 8. 不做的

- TUI 代码本身不重构（只拆分 crate，功能不变）
- i-rs-api 不修改（保留现有 70 工具端点）
- 不引入新的第三方服务（Redis Pub/Sub 等）
- 不改变现有 SessionManager 和 Storage 接口（已在近期重构中稳定）
