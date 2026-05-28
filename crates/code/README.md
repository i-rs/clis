# i-rs-code

通用编码 Agent，支持 ReAct / Plan-then-Execute 模式，集成 LSP 诊断、PTY 会话、MCP 工具扩展。

## 安装

```bash
cargo install --path crates/code
```

## 配置

配置文件位于 `~/.i-rs-code/config.toml`（自动生成）：

```toml
provider = "openai"
api_key = "sk-xxx"
model = "gpt-4o-mini"

[agents.code]
model = "gpt-4o"

[agents.analyst]
provider = "anthropic"
model = "claude-sonnet-4-20250514"

[[mcp_servers]]
name = "playwright"
command = "npx @anthropic-ai/claude-code-mcp"
```

## 使用

```bash
# 交互模式
i-rs-code tui

# 单次对话（非流式）
i-rs-code chat "重构这个模块"

# 列出配置的 Agent
i-rs-code config list

# 查看版本
i-rs-code version
```

## 工具

| 工具 | 说明 |
|------|------|
| `read`/`write`/`edit` | 文件读写编辑 |
| `glob`/`grep`/`ls` | 文件搜索查找 |
| `bash` | 命令执行（沙箱保护） |
| `delete`/`rename` | 文件删除/重命名 |
| `git` | Git 操作 |
| `web_fetch`/`web_search` | 网络搜索（SerpAPI/Bing/DuckDuckGo） |
| `lsp_diagnostics`/`lsp_definition`/`lsp_references` | LSP 代码分析 |
| `pty_exec`/`pty_interrupt` | 持久化 Shell 会话 |
| `mcp_connect` | MCP 工具扩展 |
| `create_crate` | 创建新 Rust crate |

## 模式

- **ReAct**: 标准思考-行动-观察循环（默认）
- **Plan-then-Execute**: 对复杂/重型任务，先自动生成计划再执行

## 搜索配置

在 `config.toml` 中配置搜索 API key：

```toml
search_provider = "serpapi"   # 或 "bing"
search_api_key = "your-key"
```

未配置时自动回退到 DuckDuckGo。

## 搜索

i-rs-code 属于 [i-rs 项目](https://github.com/mankong/i-rs) 的编码 Agent 组件。

在 ~/.i-rs-code/ 目录下可查看会话记录、工具记忆、配置等。
