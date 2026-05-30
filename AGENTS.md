# AGENTS.md - i-rs 项目

## Project Overview

Rust monorepo 包含 **75+ crate** + **3 个客户端**，覆盖三类产品形态：

| 类别 | 组件 | 说明 |
|------|------|------|
| **CLI 工具** | 70 个 `i-rs-{name}` | 个人数据管理命令行工具 |
| **智能助理** | `i-rs-claw` | TUI AI 助理，带 Dashboard 可观测性扩展 |
| **服务器** | `i-rs-api` | REST API 服务器 (Axum) |
| **协议服务** | `i-rs-mcp` | MCP 协议服务器 (JSON-RPC over stdio) |
| **共享库** | `i-rs-core` | 所有 crate 的基础库 |
| **客户端** | `dashboard-ui` | i-rs-claw 内嵌 Web Dashboard |
| **客户端** | `IrsClawApp` | 原生客户端 (macOS / iPad / iOS, SwiftUI) |
| **客户端** | `IrsClawMiniProgram` | 微信小程序客户端 |

**Current state:** `cargo check` — 0 errors, 0 warnings. `cargo test -p i-rs-claw` — 206 tests, 0 failed. `cargo test -p i-rs-core` — 21 tests. `cargo test -p i-rs-api` — 32 integration tests.

## Project Structure

```
i-rs-clis/
├── crates/
│   ├── clis/               # [CLI 工具] 70 个 i-rs-{name}
│   │   └── i-rs/           # Meta CLI (统一入口)
│   ├── claw/               # [智能助理] i-rs-claw TUI AI assistant
│   │   ├── src/            # 67 个源文件
│   │   ├── prompts/        # LLM 系统提示词
│   │   └── dashboard-ui/   # Dashboard 前端 (嵌入资源)
│   ├── cli-api/            # [REST API] i-rs-api Axum 服务器
│   ├── mcp/                # [MCP 协议] i-rs-mcp 服务器
│   └── core/               # [共享库] i-rs-core
├── apps/                   # [客户端] 多端应用
│   ├── IrsClawApp/         # 原生客户端 (macOS / iPad / iOS, SwiftUI)
│   └── IrsClawMiniProgram/ # 微信小程序客户端
├── docs/                   # VitePress 文档
├── skills/                 # AI 技能文档 (70 CLI crates)
├── extensions/             # 浏览器扩展 + Native Messaging
├── .github/workflows/
│   ├── release.yml         # cargo-dist 自动发布
│   └── check.yml           # CI: check + clippy + fmt
├── deny.toml
├── rust-toolchain.toml
├── Cargo.lock
├── Cargo.toml
├── README.md
├── SPEC.md
└── AGENTS.md               # 本文
```

---

## 1. CLI 工具 (70 个 i-rs-{name})

以下规范仅适用于 `crates/clis/i-rs-{name}` 这类标准 CLI 工具。

### CLI Tools Summary

| Tool | Description | Special Commands |
|------|-------------|-----------------|
| i-rs-server | Server management | suggest |
| i-rs-password | Password management | - |
| i-rs-bookmark | Bookmark management | - |
| i-rs-note | Note management | - |
| i-rs-domain | Domain expiry tracking | - |
| i-rs-remind | Event reminders | done |
| i-rs-weight | Weight tracking | chart, stats |
| i-rs-height | Height tracking | stats |
| i-rs-mood | Mood tracking | calendar |
| i-rs-sleep | Sleep tracking | stats |
| i-rs-todo | Todo tracking | done |
| i-rs-water | Water intake tracking | - |
| i-rs-step | Step counting | - |
| i-rs-dose | Medicine dosage | - |
| i-rs-cycle | Menstrual cycle | - |
| i-rs-sit | Sedentary reminder | - |
| i-rs-allergy | Allergy tracking | - |
| i-rs-cal | Calorie estimation | - |
| i-rs-fast | Fasting tracking | - |
| i-rs-exercise | Exercise tracking | stats |
| i-rs-run | Running records | plan, stats |
| i-rs-cycling | Cycling tracking | stats |
| i-rs-habit | Habit tracking | checkin, streak |
| i-rs-sub | Subscription tracking | - |
| i-rs-bestby | Best-by date tracking | - |
| i-rs-ledger | Accounting | - |
| i-rs-recur | Recurring expenses | - |
| i-rs-budget | Budget management | expense, stats |
| i-rs-invest | Investment tracking | stats |
| i-rs-debt | Debt management | pay, stats |
| i-rs-invoice | Invoice management | stats |
| i-rs-tax | Tax records | stats |
| i-rs-goal | Savings goals | deposit, milestone, stats |
| i-rs-kv | Key-value storage | - |
| i-rs-keys | API key management | - |
| i-rs-meal | Meal tracking | - |
| i-rs-pig | Craving tracking | - |
| i-rs-grocery | Grocery list | purchase, clear |
| i-rs-tick | Duration tracking | - |
| i-rs-spark | Inspiration capture | - |
| i-rs-want | Wish list | - |
| i-rs-gift | Gift planning | stats |
| i-rs-movie | Movie tracking | stats |
| i-rs-podcast | Podcast tracking | stats |
| i-rs-contact | Contact management | remind, stats |
| i-rs-car | Vehicle management | fuel, maintain, stats |
| i-rs-project | Project management | milestone, stats |
| i-rs-article | Article tracker | stats |
| i-rs-read | Reading tracker | stats |
| i-rs-quote | Quote collection | - |
| i-rs-snippet | Code snippet manager | - |
| i-rs-vocab | Vocabulary learning | quiz, stats |
| i-rs-birthday | Birthday tracking | stats |
| i-rs-event | Event management | stats |
| i-rs-time | Time tracking | start, stop, report, stats |
| i-rs-deploy | Deployment tracking | rollback, stats |
| i-rs-vision | Vision tracking | stats |
| i-rs-sheet | Bedsheet replacement | - |
| i-rs-toothbrush | Toothbrush replacement | - |
| i-rs-towel | Towel replacement | - |
| i-rs-bed | Mattress/pillow replacement | - |
| i-rs-ac | AC cleaning | - |
| i-rs-filter | Filter cleaning | - |
| i-rs-purify | Water purifier filter | - |
| i-rs-appliance | Appliance management | stats |
| i-rs-plant | Plant care | water, stats |
| i-rs-feedpet | Pet feeding | - |
| i-rs-petbath | Pet bathing | - |
| i-rs-walkdog | Dog walking | - |
| i-rs-aqua | Aquarium maintenance | - |

### Crate Structure (CLI Tools Only)

```
crates/clis/i-rs-{name}/
├── src/
│   ├── main.rs           # CLI entry: Cli::parse() + exit_on_error!
│   ├── commands/         # add, delete, get, list, update, example, skill
│   │   └── skill.rs     # ONE LINE: i_rs_core::skill_command!("i-rs-xxx");
│   ├── models/           # Entity + Row (serde + tabled + BTreeMap)
│   ├── storage/          # ONE LINE: i_rs_core::create_store!(XxxStore, "xxx");
│   └── presentation/     # render_table() + custom format
├── Cargo.toml
└── README.md
```

### New CLI Crate Workflow

1. `mkdir -p crates/clis/i-rs-{name}/src/{models,storage,commands,presentation}`
2. Create `Cargo.toml` with minimal deps (no `dirs`/`serde_json`/`tokio`/`reqwest`)
3. Create source files following the pattern above
4. Create `README.md`, `docs/crates/i-rs-{name}/`, `skills/i-rs-{name}/SKILL.md`
5. Add to workspace `Cargo.toml` members
6. Update `docs/.vitepress/config.ts` sidebar
7. `cargo check` (0 errors, 0 warnings)

### Common Patterns (CLI Tools)

- **Storage**: `BTreeMap<String, Entity>` (NOT HashMap)
- **CRUD naming**: `add_entry`, `remove_entry`, `get_entry`, `get_entry_mut`
- **Error handling**: `exit_on_error!` macro in main.rs, `anyhow::Result` elsewhere
- **CLI framework**: clap with derive macro, kebab-case params
- **Output**: `render_table()` for tables, `output_list/output_item` for JSON
- **JSON output**: All commands support `--json` global flag
- **Password/keys**: OS keychain (keyring crate), NEVER in JSON
- **Data location**: `~/.i-rs/data/` (override with `CONFIG_DIR` env var)
- **Date handling**: chrono with `ts_seconds` serde format
- **Cargo.lock**: MUST be committed
- **No unwrap()**: Use `expect("msg")` or proper error handling

---

## 2. i-rs-claw (TUI 智能助理)

`crates/claw/` — 一个 **TUI AI 助理**，深度集成 i-rs CLI 工具集，支持多模型、多 Agent、MCP 工具扩展。

### 依赖特征

- **Cargo.toml 核心依赖**: `ratatui`, `crossterm`, `tokio`, `serde_json`, `reqwest`, `clap`, `dirs`, `toml`, `uuid`, `owo-colors`, `async-trait`, `base64`
- **可选 Dashboard**: `axum`, `tower-http`, `rust-embed` (feature = `dashboard`)
- **可选 Gateway**: `gateway-telegram`, `gateway-wechat` (feature gates)
- **不需要** `i-rs-core` 依赖（直接调 CLI 二进制进程）

### 源码结构

```
crates/claw/src/
├── main.rs            # 入口: CLI 子命令分发 (tui/config/chat/version)
├── cli.rs             # clap CLI 定义 + 子命令处理
├── app.rs             # App 状态结构体 (UI 消息列表、输入、会话列表等)
├── config.rs          # ~/.i-rs/claw/config.toml 解析 + Agent 配置
├── tui.rs             # TUI 主循环 (事件处理 + LLM 事件分发)
├── ui/                # ratatui 渲染 (模块化拆分)
│   ├── mod.rs
│   ├── chat.rs
│   └── utils.rs
├── llm.rs             # LlmEvent 枚举、流式事件类型
├── providers/         # LLM 提供者 (OpenAI/Anthropic/Ollama/Zhipu)
│   ├── mod.rs
│   ├── sse.rs
│   ├── openai.rs
│   ├── anthropic.rs
│   ├── ollama.rs
│   └── zhipu.rs
├── mcp.rs             # MCP 协议客户端 (注册表 + 工具发现)
├── tools/             # 内置工具注册表
│   ├── mod.rs         # ToolRegistry + ToolContext
│   ├── i_rs.rs        # i-rs CLI 工具包装
│   ├── mcp_tools.rs   # MCP 工具Schema转换
│   ├── web_search.rs  # 网页搜索工具
│   ├── file_ops.rs    # 文件操作工具
│   ├── delegate.rs    # 子 Agent 委托工具
│   ├── chart_tool.rs  # 图表工具
│   ├── skill_tool.rs  # 技能工具
│   ├── user_memory.rs # 用户记忆工具
│   ├── search_tools.rs# 搜索工具
│   ├── semantic_search.rs
│   ├── chat_search.rs # 会话搜索
│   └── vision_tool.rs # 视觉工具
├── core/
│   ├── mod.rs         # AppCore (统一的运行时状态)
│   ├── engine/        # chat_loop + 消息构建 + smart_compress
│   │   ├── mod.rs
│   │   ├── builder.rs
│   │   └── execution.rs
│   ├── context.rs     # ContextManager (自适应 token 压缩)
│   ├── executor.rs    # ToolExecutor (超时/重试/并行)
│   └── orchestrator.rs # Plan-then-Execute (实验性/配置可选)
├── router.rs          # TaskRouter (多模型路由)
├── session.rs         # SessionManager + 会话状态机
├── semantic.rs        # EmbeddingSearch (TF-IDF + 向量语义搜索)
├── memory.rs          # CrossSessionMemory (跨会话用户记忆)
├── skill_store.rs     # 技能文档存储
├── tool_cache.rs      # 工具文档缓存 (skill teach)
├── completion.rs      # 输入补全
├── theme.rs           # 主题定制 (theme.json)
├── plugin.rs          # 插件自动发现
├── convstore.rs       # 会话搜索存储
├── stats/             # Token 用量统计
│   ├── mod.rs
│   └── store.rs
├── utils.rs           # 工具函数 (ansi 处理, JSON 前缀查找, smart_truncate)
├── gateway/           # 社交平台集成 (Telegram/WeChat)
│   ├── mod.rs
│   ├── telegram.rs
│   └── wechat.rs
└── dashboard/         # Web Dashboard
    ├── mod.rs
    ├── routes.rs
    └── assets.rs
```

### 核心架构

```
用户输入 → AppCore.build_messages_for()
              ↓
          engine::build_messages()  → 构建 API 消息列表 (系统提示词 + 历史 + 工具索引)
              ↓
          engine::chat_loop()       → ReAct 循环 (stream→tool_call→result→loop→done)
              ↓
          LlmEvent 流               → Token / ToolExecuted / Error / Done
              ↓
          TUI 渲染 (ui.rs)           → 实时显示
```

**关键数据流**:
- `Config` → `AgentConfig` (每个 Agent 可独立配置 provider/model/tools)
- `AppCore` 是全局单例，持有 `SessionManager`, `AgentRuntimeStore` (每 Agent 的 memory/tool_cache/skill_store/mcp)
- `chat_loop` 是纯 ReAct: stream → 收到 tool_calls → 并行执行 → 结果塞回消息 → 再次请求 LLM → 直到 LLM 返回文本

### Config 文件 (~/.i-rs/claw/config.toml)

```toml
provider = "openai"
api_key = "sk-xxx"
base_url = "https://api.openai.com/v1"
model = "gpt-4o-mini"
# execution_mode = "PlanThenExecute"  # 实验性，默认 ReAct

[agents.chatgpt]
model = "gpt-4o"

[agents.claude]
provider = "anthropic"
api_key = "sk-ant-xxx"

[sub_agents.analyst]
model = "o3-mini"
capabilities = ["数据分析", "代码生成"]

[[mcp_servers]]
name = "playwright"
transport_type = "stdio"
command = "npx @anthropic-ai/claude-code-mcp"
```

### 开发规范

1. **不要用 `i-rs-core`** — claw 直接调 CLI 二进制进程 (`std::process::Command`)
2. **所有异步操作走 tokio** — `tokio::spawn` + `mpsc` 通道
3. **LLM 流式事件** — 通过 `LlmEvent` 枚举传递给 TUI
4. **工具添加** — 在 `tools/` 下新建文件，注册到 `ToolRegistry`
5. **测试** — `cargo test -p i-rs-claw` (206 tests, 需要 `--test-threads=1` 避免 env var 竞争)
6. **文档** — 无需 `docs/crates/` 或 `skills/`，无 README 要求
7. **Dashboard 开发** — 需要 `dashboard` feature：`cargo check --features dashboard`

---

## 3. i-rs-api (REST API 服务器)

`crates/cli-api/` 是一个 Axum 服务器，将 70 个 CLI 工具暴露为 REST API。

### 依赖特征

- 依赖所有 `i-rs-{name}` crate（直接调 service 层，不走 CLI）
- 核心栈：`axum` + `tokio` + `tower-http` + `serde_json`
- 使用 `paste` crate 进行宏元编程

### 源码结构

```
crates/cli-api/src/
├── main.rs     # 入口: Axum 服务器启动 + make_app_tools! 宏调用 (26KB)
├── api.rs      # 常量 + 辅助函数
├── store.rs    # SharedStore 线程安全包装
├── routes.rs   # 自动生成的路由模块声明 (build.rs 生成)
├── response.rs # ApiError / ApiResult 统一错误格式
├── update.rs   # merge_entry() 通用 JSON 合并/部分更新
└── routes/     # 70 个路由模块 (每 CLI 工具一个 .rs 文件)
    ├── weight.rs
    ├── mood.rs
    └── ...
```

### 关键模式

- **`make_app_tools!` 宏** — 从统一的 (field, store_type, filename) 元组列表生成 `AppState`、`load_state()`、`build_base_router()`
- **`build.rs`** — 自动生成 `routes.rs` 的模块声明
- **Service 层复用** — API 路由调用 `i_rs_{name}::service::*`，非 CLI handler
- **每工具 5 端点**: `GET /` (list), `POST /` (create), `GET /{id}` (get), `DELETE /{id}` (delete), `PATCH /{id}` (update)
- **Data 端点**: `GET /data/export`, `POST /data/import`, `DELETE /data/clear`
- **统一错误格式**: `ApiError` / `ApiResult`，JSON 结构一致
- **部分更新**: `PATCH` 方法，`merge_entry()` 支持嵌套对象深合并 + null 字段删除

### 开发规范

1. **添加新 CLI 工具后** — 在 `main.rs` 的 `make_app_tools!` 列表中添加对应条目
2. **添加新路由** — 在 `routes/` 下新建文件，`build.rs` 自动生成模块声明
3. **测试** — `cargo test -p i-rs-api` (32 integration tests)
4. **无需** `docs/crates/` 或 `skills/`
5. **不遵循** CLI crate 结构（无 `commands/` `models/` `storage/` `presentation/`）

---

## 4. i-rs-mcp (MCP 协议服务器)

`crates/mcp/` 实现 MCP (Model Context Protocol) 服务器，通过 JSON-RPC 2.0 over stdio 暴露 65 个工具。

**详见 [crates/mcp/README.md](/crates/mcp/README.md)**。关键点：

- `make_mcp_tools!` 宏从统一的元组列表生成工具注册
- 线程安全 `SharedStore<T>` (Arc<RwLock<T>>)
- 运行时 JSON 反射 (`store_values()` / `store_get()`)
- 不遵循 CLI crate 结构

---

## 5. i-rs-core (共享库)

所有 crate 的公共基础库。

**详见 [crates/core/README.md](/crates/core/README.md)**。关键点：

- **不能依赖**任何 `i-rs-*` crate
- 提供 `Storage<T>`, `create_store!`, `skill_command!`, `exit_on_error!`
- 21 unit tests
- 详见本文前面 CLI 工具的 "Common Patterns" 章节

---

## 6. IrsClawMiniProgram (微信小程序客户端)

`apps/IrsClawMiniProgram/` — 微信小程序客户端，提供 i-rs 个人数据管理工具的移动端体验。

### 源码结构

```
apps/IrsClawMiniProgram/
├── app.json              # 小程序配置
├── app.ts                # 应用入口
├── miniprogram/          # 小程序主包
│   ├── components/       # 公共组件
│   ├── pages/            # 页面
│   ├── utils/            # 工具函数
│   └── services/         # API 调用层 (调用 i-rs-api)
├── cloud/                # 云开发 (可选)
├── project.config.json   # 项目配置
└── README.md
```

### 架构要点

- **数据来源**: 所有数据通过调用 `i-rs-api` REST API 获取
- **UI 框架**: 微信原生小程序框架 (WXML + WXSS + TypeScript)
- **用户绑定**: 通过微信 openid 关联 i-rs 用户数据
- **离线能力**: 本地缓存热点数据，支持弱网环境查看
- **API 调用**: 封装 `wx.request` 为统一 Service 层，支持 token 认证

### 开发规范

1. **不依赖 Rust crate** — 纯前端项目，无 Rust 编译产物
2. **API 优先** — 功能变更先在 `i-rs-api` 确认可用，再实现小程序端
3. **组件复用** — 公共组件放在 `miniprogram/components/` 目录
4. **状态管理** — 使用全局 `app.ts` + 页面局部状态，避免引入第三方状态库
5. **适配** — 支持 iPad 小程序 (screen size 适配 + rpx 单位)
6. **测试** — 微信开发者工具真机调试 + 模拟器预览

---

## 7. 多端适配规范

项目提供多个客户端形态，不同端共享同一套数据后端 (`i-rs-api`)，但交互方式和能力各有侧重。

### 客户端矩阵

| 客户端 | 平台 | UI 框架 | 核心场景 | 数据源 |
|--------|------|---------|---------|--------|
| `i-rs-claw` | 终端 (macOS/Linux) | ratatui (TUI) | AI 对话 + 工具调用 | CLI 二进制 + MCP |
| `dashboard-ui` | 浏览器 (Web) | React (嵌入 claw) | 数据可视化 + 管理 | i-rs-api |
| `IrsClawApp` | macOS/iPad/iOS | SwiftUI | 原生 AI 助理 | i-rs-api |
| `IrsClawMiniProgram` | 微信 (iOS/Android) | WXML + WXSS | 移动端快速查询 + 录入 | i-rs-api |

### 适配原则

1. **API 一致性** — 所有客户端通过 `i-rs-api` 统一的 REST 端点读写数据，禁止客户端直连存储文件
2. **功能子集** — 小程序和原生 App 为 CLI 功能子集，优先支持高频 CRUD + 核心统计命令
3. **UI 映射规则**
   - CLI 命令 → 移动端页面或操作按钮
   - CLI 参数 → 移动端表单字段
   - CLI 表格输出 → 移动端列表/卡片视图
   - CLI 统计命令 → 移动端图表展示
4. **离线策略**
   - 小程序: localStorage 缓存，启动时同步
   - 原生 App: CoreData (iOS) / UserDefaults (macOS)
   - claw: 无离线需求 (终端持续联网)
5. **权限模型** — 所有客户端统一使用 `i-rs-api` 的 token 认证

---

## 跨类别规范

### Build & Development

```bash
# 全量编译
cargo check --workspace    # 0 errors, 0 warnings

# 测试各 crate
cargo test -p i-rs-core
cargo test -p i-rs-api
cargo test -p i-rs-claw -- --test-threads=1  # 避免 env var 竞争

# 完整 CI 检查
cargo clippy --workspace -- -D warnings
cargo fmt --all --check
cargo deny check
```

### VitePress Documentation

`docs/` 使用 VitePress。不同类型 crate 的文档要求不同：

| 类别 | 需创建文档 | 路径 |
|------|-----------|------|
| CLI 工具 | `index.md`, `usage.md`, `examples.md`, `test.md` | `docs/crates/i-rs-{name}/` |
| claw | **不需要** | - |
| cli-api | **不需要** | - |
| mcp | **不需要** | - |
| core | `index.md`（API 参考） | `docs/crates/i-rs-core/` |

### Skills

`skills/` 目录存储 AI 技能文档：

| 类别 | 需创建 Skill |
|------|-------------|
| CLI 工具 | 每个 crate 一个 `skills/i-rs-{name}/SKILL.md` |
| claw | **不需要** |
| cli-api | **不需要** |
| mcp | **不需要** |

### Workspace Cargo.toml

添加新 crate 到 `members` 数组（根目录 `Cargo.toml`）。

### 发布

```bash
# 更新版本号 (根 Cargo.toml workspace.package.version)
git tag v0.0.x
git push origin v0.0.x
```

CI (cargo-dist) 自动构建并发布到 GitHub Releases / npm / Homebrew。

### 关键约定（所有 crate 通用）

- **Workspace deps**: 所有依赖在根 `Cargo.toml`，crate 用 `.workspace = true`
- **Build**: `cargo check` 必须 0 errors + 0 warnings
- **No unwrap()**: 用 `expect("message")` 或 proper error handling
- **Cargo.lock**: 必须提交（reproducible builds）

### 重要文件索引

- `SPEC.md` — 项目详细规范
- `AGENTS.md` — 本文件（AI 开发工作流参考）
- `Cargo.toml` — Workspace 配置
- `rust-toolchain.toml` — 固定 Rust 工具链
- `deny.toml` — cargo-deny 许可/安全配置
- `docs/.vitepress/config.ts` — VitePress 侧边栏配置
- `.github/workflows/check.yml` — CI
- `.github/workflows/release.yml` — 发布自动化
- `crates/claw/src/` — claw 源码（67 个源文件，最大 crate）
- `crates/claw/prompts/system.md` — LLM 系统提示词
- `crates/cli-api/src/update.rs` — 通用 JSON 合并/部分更新工具
- `crates/mcp/src/main.rs` — MCP 服务器入口
- `crates/core/src/` — 共享库源码
- `apps/IrsClawMiniProgram/` — 微信小程序客户端源码
