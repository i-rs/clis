# Server 后续工作

**日期**: 2026-06-07 | **状态**: Roadmap

## 1. routes.rs 拆分

**现状**: `server/routes.rs` — 1700 行单体文件，包含所有 handler。

**目标**: 按接口领域拆分为独立文件。

```
server/
└── routes/
    ├── mod.rs          # 模块声明
    ├── chat.rs         # POST /api/chat, SSE streaming, reconnect
    ├── sessions.rs     # /api/sessions/* CRUD
    ├── agents.rs       # /api/agents/* CRUD
    ├── config.rs       # /api/config, /api/providers
    ├── tools.rs        # /api/tools, /api/plugins, /api/skills
    ├── stats.rs        # /api/stats
    ├── memory.rs       # /api/memory/layered, /api/memory/search
    ├── checkpoints.rs  # /api/checkpoints/*
    ├── evals.rs        # /api/evals
    ├── guardrails.rs   # /api/guardrails/check
    └── images.rs       # /api/images/{filename}
```

拆分后 `app.rs` 路由注册改为 `crate::server::routes::chat::chat` 形式。

---

## 2. ConfigStore DB backends

**现状**: `crates/claw-core/src/storage/config_store.rs` 有 File 实现和 Empty stub。SQL/Mongo/Redis 返回空数据。

**目标**: 所有 5 个 backend 实现真实 CRUD。

| Backend | agent_configs | provider_configs | dashboard_users | mcp_server_configs | app_settings |
|---------|:---:|:---:|:---:|:---:|:---:|
| File | ✅ | ✅ | ✅ | ✅ | ✅ |
| SQLite | ❌ | ❌ | ❌ | ❌ | ❌ |
| MySQL | ❌ | ❌ | ❌ | ❌ | ❌ |
| PostgreSQL | ❌ | ❌ | ❌ | ❌ | ❌ |
| MongoDB | ❌ | ❌ | ❌ | ❌ | ❌ |
| Redis | ❌ | ❌ | ❌ | ❌ | ❌ |

**实施路径**:
1. SQL backends: 给 `define_sql_stores!` 宏增加 5 张新表 + 生成对应 trait impl
2. MongoDB: 添加 5 个 collection 的 CRUD 操作
3. Redis: 添加对应 key pattern 的读写

---

## 3. Provider/User/MCP/Settings API

**现状**: `GET /api/providers` 只读（从 config.toml）。dashboard users 只能用静态 config.toml 配置。MCP servers 和 app settings 无 API。

**新增端点**:

| 端点 | 方法 | 说明 |
|------|------|------|
| `/api/providers` | POST | 创建 provider |
| `/api/providers/{name}` | PUT/DELETE | 更新/删除 provider |
| `/api/users` | GET/POST | 多用户管理 |
| `/api/users/{id}` | DELETE | 删除用户 |
| `/api/mcp` | GET/POST | MCP server 配置管理 |
| `/api/mcp/{name}` | PUT/DELETE | 更新/删除 MCP server |
| `/api/settings` | GET/PUT | 全局 app_settings K/V |
| `/api/settings/{key}` | GET/PUT/DELETE | 单项设置 |

**File backend**: 写入 ConfigStore JSON 文件 + 同步 config.toml
**DB backend**: 仅写 ConfigStore，config.toml 不再维护 agent/provider/mcp

---

## 4. `/api/data/{tool}` 端点

Dashboard 直接暴露 i-rs 70 个数据工具的 CRUD，使 Web/App 客户端不依赖 CLI。

**两种实现路径**:

| 方案 | 做法 | 适用 |
|------|------|------|
| 直接调 service 层 | 依赖 i-rs-{name} crate，调 service 函数 | 单体部署 |
| 代理到 i-rs-api | 反向代理到 i-rs-api 进程 | 微服务 |

**当前推荐**: 方案 A（所有 CLI crate 已在 workspace 中）。

```
GET    /api/data/{tool}           → list
POST   /api/data/{tool}           → create
GET    /api/data/{tool}/{id}      → get
DELETE /api/data/{tool}/{id}      → delete
PATCH  /api/data/{tool}/{id}      → update
```

---

## 5. 测试修复

**现状**: 15 个 dashboard 测试因 tokio runtime 嵌套失败。

**根因**: `sync_block_on` 在 `#[tokio::test]` 上下文中使用 `block_in_place(|| SHARED_RUNTIME.block_on(f))`，测试结束 runtime 被 drop 时报错 "Cannot drop a runtime"。

**修复方案**:
1. 在 `SessionManager::with_storage` 增加 `with_storage_no_async()` 跳过 tokio
2. 或：`test_core()` 使用空 session 数据，不调用 async 加载
3. 或：使用 `#[test]` + 独立线程 runtime 模式（已在 middleware 测试中采用）

---

## 6. useChatStream state 改进

**现状**: `Chat.tsx` 中 streaming 期间的 `renderTick` 已移除，改用 `useState` 驱动。但仍使用 `stateRef.current` 快照返回 `state` 值 — 这是同步快照，不是响应式。

**目标**: `state` 直接从 `useState` 返回，不经过 ref 快照。

---

## 7. 前端 CSS Modules 化

**现状**: 仅 `ChatInput` 使用 CSS Module。`styles.css` 仍是 1968 行全局样式。

**目标**: 逐步迁移组件到 CSS Modules:
1. `MessageBubble` → `MessageBubble.module.css`
2. `StreamingBubble` → `StreamingBubble.module.css`
3. `ToolCallCard` → `ToolCallCard.module.css`
4. `MarkdownRenderer` → `MarkdownRenderer.module.css`

---

## 8. 前端 Data 页面

**现状**: Dashboard 没有 i-rs 数据工具看板页面。

**目标**: 新增 `/data` 路由和 DataPage 组件，对接 `/api/data/{tool}`。
- 70 个工具的列表/详情/创建/编辑
- 图表显示（weight/sleep/mood 等的趋势图）
- 响应式表格

---

## 优先级

| 优先级 | 项目 | 工作量 |
|--------|------|--------|
| P0 | routes.rs 拆分 | 1h |
| P0 | 测试修复 | 2h |
| P1 | ConfigStore DB backends | 3d |
| P1 | Provider/User/MCP/Settings API | 2d |
| P1 | `/api/data/{tool}` | 2d |
| P2 | CSS Modules 化 | 1d |
| P2 | useChatStream state 改进 | 0.5h |
| P2 | Data 页面 | 2d |
