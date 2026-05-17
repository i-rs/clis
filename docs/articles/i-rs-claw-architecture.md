# i-rs-claw 架构解析 — AI 终端助理的内部结构

## 总览

i-rs-claw 是一个 Rust TUI 应用，架构设计围绕三个核心问题展开：

1. **如何让 AI 理解 70 个 CLI 工具？** → 动态 skill teach 机制
2. **如何让 AI 和 CLI 工具协作？** → 工具抽象层 + 子进程调度
3. **如何在终端中呈现流畅的 AI 交互？** → 异步事件驱动的 TUI 架构

```
┌─────────────────────────────────────────────────────────┐
│                       main.rs                           │
│                                                         │
│  ┌──────────────────┐    ┌──────────────────────────┐   │
│  │     ui.rs        │    │        llm.rs            │   │
│  │  Ratatui TUI     │    │  LLM streaming client    │   │
│  │  事件循环        │◄──►│  工具调用调度            │   │
│  │  布局管理        │    │  上下文管理              │   │
│  └────────┬─────────┘    └────────────┬─────────────┘   │
│           │                           │                 │
│           ▼                           ▼                 │
│  ┌──────────────────────────────────────────────────┐   │
│  │                  app.rs                          │   │
│  │           应用状态 + 消息路由                     │   │
│  └─────────┬───────────────────────────┬───────────┘   │
│            │                           │               │
│  ┌─────────▼──────┐    ┌───────────────▼───────────┐   │
│  │   config.rs    │    │       tools/              │   │
│  │   TOML 配置    │    │  ┌─────────────────────┐  │   │
│  │   工具启用列表 │    │  │  mod.rs (ToolRegistry)│  │   │
│  └────────────────┘    │  │  i_rs_cmd.rs (CLI)   │  │   │
│                        │  │  search.rs (语义搜索) │  │   │
│  ┌────────────────┐    │  │  chart_tool.rs(图表)  │  │   │
│  │   session.rs   │    │  └─────────────────────┘  │   │
│  │   会话持久化   │    └───────────────────────────┘   │
│  └────────────────┘                                    │
│  ┌────────────────┐    ┌──────────────────────────┐   │
│  │   provider.rs  │    │      memory.rs           │   │
│  │  多 LLM 提供者  │    │   用户记忆持久化          │   │
│  └────────────────┘    └──────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

## 分层详解

### 第一层：TUI 事件循环（ui.rs + tui.rs）

i-rs-claw 使用 [Ratatui](https://github.com/ratatui/ratatui)（原 tui-rs 的活跃分支）构建终端界面。约 60k 行 UI 代码覆盖了：

- **对话区** — 流式渲染 LLM 回复，支持 Markdown、代码块、工具调用面板
- **输入栏** — 多行输入支持、光标移动、Tab 补全、历史导航
- **标题栏** — 实时显示模型名称、工具启用数、Token 用量
- **状态栏** — 快捷键提示、操作反馈、语音输入状态
- **会话侧边栏** — 历史会话列表、切换、删除、搜索
- **调试面板** — HTTP 请求体/响应体查看、Token 计数

事件循环是异步的，基于 `crossterm` 的原始模式事件流。所有 I/O 操作（LLM 请求、CLI 子进程）都在后台线程执行，通过通道将结果送回主循环。

### 第二层：LLM 集成（llm.rs + provider.rs）

支持任意 OpenAI 兼容 API（OpenAI、DeepSeek、OpenRouter、Groq 等），通过 `provider.rs` 抽象为统一接口：

```rust
trait LLMProvider {
    fn chat_stream(&self, messages: &[Message], tools: &[ToolDef])
        -> Result<StreamReceiver>;
}
```

关键特性：
- **SSE 流式推送** — 实时显示 token 生成过程
- **工具调用拦截** — 从流中检测 `tool_calls`，本地执行后注入结果
- **系统提示组装** — 动态注入工具索引、热工具列表、用户记忆、SKILL 文档
- **自动重试** — 网络错误时自动重试，避免一次失败丢掉整个对话

### 第三层：工具系统（tools/）

这是 i-rs-claw 最独特的架构设计。工具被抽象为统一的 `ClawTool` trait：

```rust
pub trait ClawTool {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, args: &Value) -> Result<String, String>;
}
```

三种工具类型：

| 类型 | 注册方式 | 示例 |
|------|----------|------|
| **CLI 工具** | 从 TOOL_INDEX 自动注册 | `i_rs(tool="weight", command="add", args=["...", "75"])` |
| **内置工具** | 手动实现 ClawTool | `search_tools()`, `update_user_memory()`, `chart()` |
| **MCP 工具** | 通过 MCP 协议动态加载 | 外部 MCP 服务器暴露的工具 |

#### CLI 工具执行器（i_rs_cmd.rs）

当 AI 调用 `i_rs(tool="weight", command="add", ...)` 时：

1. `ToolRegistry` 查找 `i_rs` 工具
2. `IrsTool::execute()` 构造 `i-rs weight add --json ...` 命令
3. 通过 `std::process::Command` 启动子进程
4. 30 秒超时控制，防止死锁
5. 解析 stdout/stderr，返回成功结果或错误信息

#### 动态工具索引

TOOL_INDEX 不再硬编码。i-rs-claw 从每个 CLI 工具的 `--help` 中动态发现其能力，按分类组织：

```rust
pub const TOOL_INDEX: &[(&str, &str, &str)] = &[
    ("weight", "体重管理：记录、查看、统计体重数据", "健康管理"),
    ("ledger", "记账：记录收支、查看流水", "财务管理"),
    ("todo", "待办事项：添加、完成、列出待办", "任务与习惯"),
    // ...
];
```

用户可以通过 TUI 配置界面自由启用/禁用工具，禁用的工具不会出现在 AI 的工具列表中，避免认知负担和 token 浪费。

### 第四层：Skill Teach 管道

这是 i-rs-claw 的"杀手锏"——AI 不是硬编码工具用法，而是动态学习：

```
用户说 "记录体重" 
  → AI 查看工具索引，选择 weight
  → AI 调用 i_rs(tool="weight", cmd="skill", args=["teach"])
  → 返回结构化的教学文档（命令、参数、示例）
  → AI 根据文档调用 i_rs(tool="weight", cmd="add", args=["2026-05-16", "75"])
  → 成功！
```

这个过程每次对话只做一次——结果会被缓存到磁盘，后续对话直接使用缓存的 skill teach 结果，避免重复加载。

### 第五层：会话与记忆

#### 会话管理（session.rs）

每次对话自动创建会话文件（JSON 格式），包含：
- 消息历史（角色、内容、工具调用记录）
- 元数据（时间戳、Token 用量、模型名称）
- skill teach 缓存（避免重复加载）

支持会话的 CRUD 操作：新建、切换、重命名、删除。

#### 上下文压缩

当对话过长时，自动执行智能压缩：
- **保留** skill teach 文档、用户记忆、系统提示
- **摘要** 历史对话的中间轮次
- **压缩率** 通常 5k → 1.5k tokens

#### 用户记忆（memory.rs）

AI 可以将用户信息持久化到本地 JSON 文件：
- 用户称呼、兴趣、习惯
- 每次对话自动加载
- 跨会话持久，无需重复告知

### 第六层：扩展接口

#### MCP 协议支持（mcp.rs）

实现了 Model Context Protocol（MCP）客户端，可以连接任意 MCP 服务器：
- 动态发现 MCP 工具
- JSON-RPC 2.0 通信
- stdio 传输协议

#### 语义搜索（semantic.rs）

基于 TF-IDF 的轻量语义搜索，无需外部向量数据库：
- 对历史会话建立倒排索引
- 支持关键词搜索和 TF-IDF 排序
- 所有计算在本地完成，无需网络

## 数据流全景

```
用户输入 "记录体重 75kg 并查看今天心情"
    │
    ▼
ui.rs: 捕获输入，发送到 app.rs
    │
    ▼
app.rs: 构造消息对象，追加到历史
    │
    ▼
llm.rs: 发送完整历史 + 系统提示（含工具索引）
    │
    ▼
LLM API: 返回流式响应
    │
    ├── 文本 token → 实时渲染到对话区
    │
    └── tool_calls: [tool="weight", cmd="add"]
        │
        ▼
    tools/i_rs_cmd.rs: 执行 i-rs weight add --json 2026-05-16 75
        │
        ▼
    子进程完成 → 结果注入到 LLM 上下文
        │
        ▼
    LLM API: 继续生成回复（基于工具结果）
        │
        ▼
    ui.rs: 渲染最终回复（含工具调用面板）
```

## 技术栈

| 层 | 技术 | 用途 |
|----|------|------|
| UI | [Ratatui](https://github.com/ratatui/ratatui) + [Crossterm](https://github.com/crossterm-rs/crossterm) | 终端界面和事件处理 |
| LLM | OpenAI 兼容 API | 流式聊天完成 |
| 运行时 | Tokio（部分） + std threads | 异步事件和子进程管理 |
| 存储 | serde_json + 本地 JSON 文件 | 数据持久化 |
| 搜索 | TF-IDF（自定义实现） | 语义搜索 |
| 协议 | MCP (JSON-RPC 2.0) | 外部工具扩展 |
| 构建 | Cargo workspace | 75 crate 单体仓库 |

## 设计原则

1. **本地优先** — 除了 LLM API 调用，所有操作在本地完成。数据不离机。
2. **透明可审计** — AI 的每一个工具调用都在界面上可见，没有隐藏操作。
3. **容错设计** — 子进程超时、LLM 网络中断、配置错误——每个环节都有明确的错误处理和恢复路径。
4. **按需加载** — 系统提示不预加载所有 skill teach 文档，只在 AI 需要时才通过 `skill teach` 加载，最大化上下文可用性。
5. **渐进式对齐** — 70 个 CLI 工具不是一刀切全部启用，而是逐步对齐标准化架构，确保每个被启用的工具都有完整的 JSON 输出和一致的错误处理。
