# i-rs-code

独立通用编码 AI Agent，TUI 交互式编程助手。支持 ReAct / Plan-then-Execute 模式，集成文件编辑、LSP 诊断、Shell 执行、MCP 工具扩展。

与 [i-rs 项目](https://github.com/mankong/i-rs) 的 claw TUI 助理配合使用时，可通过 JSON-RPC over stdio 协议接受 claw 委派的代码生成任务（如创建新的 CLI 工具 crate）。

## 安装

```bash
cargo install --path crates/code
```

需要 `tui` feature（默认开启）：
```bash
cargo install --path crates/code --features tui
```

## 配置

配置文件位于 `~/.i-rs/code/config.toml`（首次运行自动生成）：

```toml
provider = "openai"
api_key = "sk-xxx"
base_url = "https://api.openai.com/v1"
model = "gpt-4o-mini"
max_rounds = 25
tool_timeout_secs = 60
```

### 环境变量

| 变量 | 说明 |
|------|------|
| `OPENAI_API_KEY` | API key（优先于配置文件） |
| `CONFIG_DIR` | 数据目录（默认 `~/.config/i-rs`） |

## 使用

```bash
# 交互式 TUI（默认）
i-rs-code tui

# 恢复上次会话
i-rs-code tui --session <session-id>

# 单次对话（stdin/stdout）
i-rs-code chat "重构这个模块"

# Agent 模式（JSON-RPC over stdio，供 claw 调用）
i-rs-code agent --task-id "uuid-123"

# 配置管理
i-rs-code config show      # 查看配置
i-rs-code config init      # 交互式设置
i-rs-code config set key value  # 设置单项

# 会话搜索
i-rs-code search "关键词"

# 查看版本
i-rs-code version
```

## TUI 快捷键

| 按键 | 功能 |
|------|------|
| `Enter` | 发送消息 |
| `Alt+Enter` | 换行 |
| `Esc` / `q` | 退出 / 关闭面板 |
| `Ctrl+C` | 取消当前生成 |
| `Ctrl+Z` | 撤销文件修改 |
| `Ctrl+T` | 转录模式（完整未截断输出） |
| `Ctrl+D` | HTTP 调试面板 |
| `[` / `]` | 选择上/下一条消息 |
| `r` | 展开/折叠选中消息的思考过程 |
| `↑` / `↓` / `PgUp` / `PgDn` | 滚动聊天 |
| `Tab` | 工具名补全 |
| `?` | 键盘快捷键面板 |

## 工具

| 工具 | 说明 |
|------|------|
| `read` / `write` / `edit` | 文件读写编辑 |
| `glob` / `grep` / `ls` | 文件搜索查找 |
| `bash` | Shell 命令执行（沙箱保护） |
| `delete` / `rename` | 文件删除/重命名 |
| `git` | Git 操作（commit/diff/log） |
| `web_fetch` / `web_search` | 网络搜索（SerpAPI/Bing/DuckDuckGo） |
| `lsp_diagnostics` / `lsp_definition` / `lsp_references` | LSP 代码分析 |
| `pty_exec` / `pty_interrupt` | 持久化 Shell 会话 |
| `mcp_connect` | MCP 工具扩展 |
| `create_crate` | 生成标准 i-rs CLI crate 模板 |
| `call_claw` | 请求 claw 协助调试 |
| `register_tool` | 输出工具定义供 claw 注册 |

## 架构

```
src/
├── main.rs              # 入口：clap 子命令分发 (tui/chat/agent/config/search)
├── app.rs               # App 状态 + AgentMessage enum
├── cli.rs               # CLI 参数定义
├── config.rs            # 配置加载/保存
├── agent/
│   ├── mod.rs           # Agent struct + ChatSession trait
│   ├── engine.rs        # ReAct 循环 (stream → tool_calls → result → loop)
│   ├── context.rs       # 上下文压缩管理
│   └── event.rs         # AgentEvent 枚举
├── provider/
│   ├── openai.rs        # OpenAI 兼容 API
│   ├── anthropic.rs     # Anthropic API
│   └── error.rs         # 结构化错误类型
├── tools/               # 15+ 工具实现
├── tui/                 # Ratatui TUI 渲染
├── protocol/            # JSON-RPC over stdio 协议
├── session.rs           # 会话持久化
├── memory.rs            # 跨会话记忆
├── diff.rs              # 差异对比/应用
├── lsp.rs               # LSP 客户端
├── pty.rs               # PTY 会话管理
├── mcp.rs               # MCP 协议客户端
├── router.rs            # 执行模式路由
├── debug.rs             # HTTP 调试日志
└── runtime.rs           # 全局单例
```

## 模式

- **ReAct**（默认）：标准思考-行动-观察循环
- **Plan-then-Execute**：对复杂任务先自动生成计划再执行

## Claw 集成

i-rs-code 可通过 JSON-RPC over stdio 协议接受 claw 委派的代码生成任务：

```bash
# claw 启动 i-rs-code 子进程
i-rs-code agent --task-id "uuid-123"

# stdin (claw → code)：task / respond / cancel
# stdout (code → claw)：token / progress / request / tool_created / done
```

典型流程：
1. claw 判断需要代码生成 → spawn `i-rs-code agent --task-id "uuid-123"`
2. claw 发送 `task` 消息（prompt + workspace 上下文）
3. i-rs-code ReAct 循环：创建文件 → 编译 → 调试 → 完成
4. i-rs-code 通过 `register_tool` 事件输出新工具定义
5. claw 接收并注册工具到 ToolRegistry

## 会话

所有会话记录保存在 `~/.i-rs/code/sessions/`。可通过 `i-rs-code search <query>` 跨会话搜索。使用 `i-rs-code tui --session <id>` 恢复历史会话。

## AgentMessage 类型

TUI 中每条消息有明确的语义类型，渲染风格各异：

| 类型 | 视觉 | 说明 |
|------|------|------|
| User | 蓝色 `User` 标签 | 用户输入 |
| Assistant | 绿色 `AI` 标签 + 灰色推理 | AI 回复 + 可折叠思考 |
| ToolResult | 黄色 `glyph name` + `└` 延续 | 工具执行结果 |
| FileEdit | 品红 `✎ path` + diff 着色 | 文件修改摘要 |
| Separator | 灰色 `── done ──` | 轮次分隔线 |
| System | 绿色/黄色总结 | 完成总结 |
