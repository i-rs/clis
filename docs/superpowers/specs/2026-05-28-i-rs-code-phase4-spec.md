# i-rs-code Phase 4: 生产级编码 Agent

&gt; **定位**: 通用编码 Agent (对标 Claude Code / Aider)
&gt; **版本**: 0.3.0
&gt; **状态**: 草案

## 1. 概述

i-rs-code 目前是一个功能完整的编码 Agent（5,117 行 Rust，32 源文件，21 测试），但缺少生产级编码助手的关键能力。Phase 4 的目标是补齐 P0–P2 差距，使其达到可日常使用的水平。

## 2. P0 关键缺失 — 必须实现

### 2.1 LSP 集成 (tower-lsp)

**现状**: 无 LSP 支持，Agent 只能通过 `grep` + `read` 理解代码。

**方案**: 进程内 LSP 客户端 (tower-lsp)，启动 rust-analyzer（或其他指定语言的 LSP server），通过 JSON-RPC 通信。

**能力**:
- `lsp_diagnostics()`: 获取文件诊断错误/警告列表 → 注入上下文供 Agent 参考
- `lsp_definition()`: 获取符号定义位置 → 替代手动 grep
- `lsp_references()`: 查找符号引用 → 理解代码影响范围
- `lsp_completion()`: 获取补全候选（非自动触发，按需查询）
- LSP 服务器管理: 自动启动/保活/重启

**架构**:
```
Agent → LspService (tower-lsp) → ChildProcess (rust-analyzer / typescript-language-server 等)
         ↓
    LspContext (缓存诊断 + 符号表)
```

**选择理由**: tower-lsp 是 Rust 最成熟的 LSP 实现，直接集成比桥接更简单。进程内通信延迟低。

### 2.2 PTY 终端 (tokio-pty + pty-process)

**现状**: `bash` 工具仅使用 `tokio::process::Command::output`，无法处理交互式命令（vim、`npm init`、`cargo watch`、`python -i`）。

**方案**: 使用 `portable-pty` 或直接基于 `tokio` + `pty-process` crate 实现伪终端。

**能力**:
- `pty_exec(command, timeout)`: 在 PTY 中执行命令并流式返回输出
- `pty_interrupt(session_id)`: 发送 Ctrl+C 终止当前进程
- `pty_write(session_id, input)`: 向运行中的进程发送输入（极少数场景）
- 终端行数/列数模拟
- 会话保持：每个工具调用间保持 shell 状态

**安全**: PTY 会话受同一白名单沙箱约束。

### 2.3 语义上下文管理

**现状**: `total_chars / 4` 估算 token，暴力截断丢弃早期 tool_call 对，无语义摘要。

**方案**: 实现多层上下文管理系统，按 token 预算分层压缩：

1. **层级 1 — 路径优化** (立即生效): 将 `[File: /path/to/project/src/main.rs]` 中的绝对路径替换为相对路径
2. **层级 2 — 摘要替换** (压缩阶段): 对丢弃的 tool_call 对，保留摘要而非直接丢弃
3. **层级 3 — 精简历史** (限额 80%+): 保留系统提示词 + 最后 N 轮对话，其余做 tiktoken / 字符估算的语义摘要
4. **层级 4 — 记忆注入** (跨会话): 从 `memory.rs` 提取相关上下文注入

**关键变更**: 将 `ContextManager` 从单纯截断改为分层压缩，token 估算改用 `tiktoken-rs` 或更准确的算法。

### 2.4 安全沙箱 (路径白名单)

**现状**: `check_path` 只拦截 `..` 和绝对路径不在 workspace 内。Bash blocklist 可被 `rm --no-preserve-root -rf /` 绕过。

**方案**:
- `BashTool`: 强制 `--chdir` 限制到 workspace 目录；拦截 `sudo`、`chmod 777`、`rm -rf /` 等。
- `filesystem` 工具: 现有白名单扩展，全部走 canonicalize 验证
- 新 `delete` 工具: 替代 `bash rm`，纳入白名单检查
- 新 `rename` 工具: 纳入白名单检查

### 2.5 测试覆盖

**现状**: 21 个测试，全部集中在 config/session/memory/convstore/router/bash，核心 engine + providers 零测试。

**目标**: 核心模块达到 70%+ 覆盖率。

**关键测试**:
- `engine` 单元测试: mock provider + mock tools，测试 ReAct 循环的各种路径（单轮、多轮、工具失败重试、提供者错误重试、上下文压缩触发）
- `provider` 集成测试: 使用 mock HTTP server 测试各 provider 的 SSE 解析
- `tools` 单元测试: 每个主要工具至少 3 个测试（正常路径、边界、错误）
- `session` 测试已有 4 个 — 补到 8 个（含分页、空会话）
- `config` 测试已有 4 个 — 补到 6 个

**测试基础设施**:
- MockLlmProvider: 实现 `LlmProvider` trait，返回可控的流式事件
- MockToolRegistry: 含少数 mock tool
- 测试辅助函数: `build_test_config()`, `ssse_chunk_to_events()` 等

## 3. P1 重要缺失 — 应实现

### 3.1 Plan-then-Execute

**现状**: 纯 ReAct 循环，无前置计划。

**方案**: 在 `router.rs` 分类基础上，Complex/Heavy 任务在 ReAct 前注入 Plan 阶段：
1. Agent 收到请求 → Router 判定复杂度
2. Heavy/Complex → 先发一个 "plan" 消息给 LLM，获取分步计划
3. 计划以结构化格式（Markdown checklist）注入为系统提示词
4. 进入 ReAct 循环，每次步骤选择参考计划

### 3.2 MCP 集成

**现状**: config.rs 声明了 `mcp_servers: Vec<McpServerConfig>` 但 `ToolRegistry::new()` 从未使用。

**方案**: 在 `tools/mcp.rs` 中实现 MCP 客户端：
- 启动 MCP server 子进程（`npx @anthropic-ai/claude-code-mcp` 等）
- 通过 JSON-RPC over stdio 发现工具列表
- 将 MCP 工具注册到 `ToolRegistry`
- 通过 JSON-RPC 调用 MCP 工具
- 连接保活 + 自动重连

### 3.3 文件工具完善

**现状**: 无独立的 `delete` / `rename` / `move` 工具，只能通过 bash 完成。

**新增工具**:
- `DeleteTool`: 安全删除文件/目录（走白名单，确认存在，可选 `--force`）
- `RenameTool`: 安全重命名文件/目录（走白名单）

### 3.4 WebSearch 鲁棒性

**现状**: 抓取 DuckDuckGo HTML，易因改版失效。

**方案**:
- 增加配置化的 SerpAPI / Bing Search API key 支持
- DuckDuckGo 作为 fallback（保持现有逻辑）
- 增加错误处理和速率限制

### 3.5 动态多模型路由

**现状**: Agent 配置静态，不会根据任务特性切换模型。

**方案**: 在 `router.rs` 中扩展 `classify_complexity` 输出包含推荐模型名：
- Simple → 快速/便宜模型 (如 gpt-4o-mini)
- Complex → 标准模型 (如 gpt-4o, claude-sonnet)
- Heavy → 强推理模型 (如 o3-mini, claude-opus)

## 4. P2 轻度缺失 — 可选实现

### 4.1 文档 / README
- `README.md` (crate 级别): 安装、配置、使用示例
- API 文档注释

### 4.2 代码质量
- 移除 `#![allow(dead_code)]`（主要影响 `agent/event.rs` 的 `FileChanged` 变体）
- 将 `protocol/handler.rs:6` 的 `unsafe` 替换为 `std::env::set_var`（Rust 1.66+ 不再是 unsafe）
- 移除未使用的 `DEFAULT_TOOL_TIMEOUT_SECS` 常量

### 4.3 TUI 优化
- 支持多行粘贴（当前 `insert_char` 每次只处理一个字符）
- 输入框支持 Ctrl+V 粘贴
- Ctrl+U 清行、Ctrl+A/E 行首尾

## 5. 实施路线

### Phase 4.1 — 基础安全 + 测试 (起点，独立无风险)
- 2.4 安全沙箱完善
- 2.5 测试覆盖 + Mock 基础设施
- 4.2 代码质量清理
- 3.3 文件工具完善

### Phase 4.2 — LSP 集成 (核心能力)
- 2.1 LSP 客户端

### Phase 4.3 — PTY + 上下文管理
- 2.2 PTY 终端
- 2.3 语义上下文管理

### Phase 4.4 — 高级 Agent 能力
- 3.1 Plan-then-Execute
- 3.2 MCP 集成
- 3.5 动态路由

### Phase 4.5 — 完善
- 3.4 WebSearch 鲁棒性
- 4.1 README/文档
- 4.3 TUI 优化
