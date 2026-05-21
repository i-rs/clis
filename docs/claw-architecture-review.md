# i-rs-claw 架构评审报告

> 日期: 2026-05-21
> 范围: crates/claw/ 全量扫描（含 dashboard / gateway / mcp 所有 feature）
> 基线: clippy --all-features 0 warnings, 217 tests pass

## 现状总结

架构设计已经很成熟：

- **AppCore** 单例持有 Config / SessionManager / AgentRuntimeStore / StatsManager
- **AgentRuntimeStore** 按 agent 隔离 memory / tool_cache / skill_store / mcp_registry
- **chat_loop** ReAct 循环（stream -> tool_call -> result -> loop -> done）
- **ToolCallExecutor** 并行执行 + 超时 + 截断
- **SessionManager** 状态机（Active / WaitingForTool / Error / Completed / Interrupted）
- **smart_compress** 按跨会话工具频率 + recency 评分保留高价值 teach pair
- **StatsManager** JSONL 追加写入 + 内存缓冲 + 定期 flush

代码质量在同类 TUI AI 助手中算上乘。以下按优先级列出改进项。

---

## P0 — 不做会出问题

| # | 问题 | 现状 | 影响 |
|---|------|------|------|
| 1 | **无 graceful shutdown** | `main_loop` 的 loop 只在 Ctrl+Q 时 break；进程被 kill 时 session 不会 flush、MCP 子进程不会被杀 | 用户丢会话、孤儿进程 |
| 2 | **Dashboard 无认证** | `dashboard/mod.rs` 绑定 `127.0.0.1:3000`，任何人可访问 `/api/chat` 发消息 | 安全风险（虽然默认 localhost） |
| 3 | **API key 明文存储** | `config.toml` 里 `api_key = "sk-xxx"` 明文；Agent 的 `api_key` 也是 | 泄露风险 |
| 4 | **SSE 流解析无 malformed data 防御** | `sse.rs:127` `serde_json::from_str` 失败后静默跳过，但 `buf.find('\n')` 对非 UTF-8 数据没有处理 | 可能死循环或丢数据 |
| 5 | **send_with_retry 不区分错误类型** | `sse.rs` 里的 retry 对所有错误都重试，包括 401（key 无效）、429（限流） | 对 401 重试 3 次浪费时间且刷日志 |

---

## P1 — 影响可靠性/用户体验

| # | 问题 | 现状 | 改进方向 |
|---|------|------|---------|
| 6 | **session 磁盘写入无错误反馈** | `session.rs` 全文 `let _ = atomic_write(...)` 静默忽略写失败 | 至少 `tracing::error!`，最好在 TUI 显示提示 |
| 7 | **memory/tool_cache 写入也静默失败** | `memory.rs:216` `let _ = atomic_write(...)` | 同上 |
| 8 | **无 LLM 请求重连/恢复** | provider 返回错误 -> `chat_loop` 直接 break 发 Error 事件 | 对 transient error（网络抖动、5xx）自动重试 1-2 次 |
| 9 | **Stats JSONL 无大小限制** | `usage.jsonl` 只在 startup/exit 时 cleanup，长期运行可能膨胀到 MB 级 | 定期 flush 时检查文件大小，超过阈值主动 prune |
| 10 | **MCP 子进程无健康检查** | `McpClient` 创建后如果子进程崩溃，下次 tool call 会得到连接错误 | 加 health check / 自动重连 |
| 11 | **Provider 重试过于粗糙** | `send_with_retry` 固定重试 3 次，无指数退避，无 jitter | 对 429 加 `Retry-After` header 解析 + 指数退避 |
| 12 | **`create_provider` 每次调用都 new reqwest::Client** | `providers/mod.rs:76-79` 每次调用创建新 Client（虽然内部有连接池） | 复用单个 Client 实例 |

---

## P2 — 架构层面可优化

| # | 问题 | 现状 | 改进方向 |
|---|------|------|---------|
| 13 | **ui.rs 62KB 单文件** | 所有渲染逻辑在一个文件，ratatui 每次 `terminal.draw` 全量重绘 | 按面板拆分为 submodules；对长消息列表加虚拟滚动（只渲染可见区域） |
| 14 | **Vec\<Value\> 消息大量 clone** | `handle_done` 里 `msgs.clone()` + `compress_api_messages` 里 `msg.clone()` | 用 `Arc<Value>` 替代消息体，减少序列化开销 |
| 15 | **`ClawError` 使用不充分** | `error.rs` 定义了 7 种错误变体，但大部分代码用 `anyhow::Result` | tool 层已用 `ClawError`，可扩展到 provider/executor 层做结构化错误 |
| 16 | **Provider trait 只返回 anyhow::Result** | `LlmProvider::stream_chat` 无法区分网络错误、认证失败、限流 | 定义 `ProviderError` enum（Transient / Auth / RateLimit / InvalidResponse） |
| 17 | **Config::validate 只 warning 不 block** | 错误的 `base_url` 格式、无效的 agent 配置只是 warning | 对明确会导致运行时失败的配置升级为 error |
| 18 | **semantic.rs TF-IDF 无增量更新** | 每次搜索重建整个索引 | 对 embedding 做增量 cache |
| 19 | **测试缺少集成层覆盖** | 217 个全是 unit test，无 chat_loop x real provider 的集成测试 | 至少加 1 个 MockProvider 端到端测试覆盖 send -> stream -> tool_call -> done 全流程 |
| 20 | **`main_loop` poll 50ms 固定间隔** | 不处理 LLM 时也每 50ms 重绘 | 用 `tokio::select!` 替代固定 poll，只在有事件时重绘 |

---

## P3 — 锦上添花

| # | 问题 |
|---|------|
| 21 | Dashboard SSE `chat_stream` 无心跳，长连接会被代理超时断开 |
| 22 | Gateway Telegram/WeChat 无消息队列，如果 LLM 响应慢，新消息会排队无反馈 |
| 23 | `tool_cache.rs` 无过期机制，skill teach 缓存永不过期 |
| 24 | `convstore.rs` 搜索是线性扫描，会话多了会慢 |
| 25 | 无 `--version` 详细信息（git commit、build date） |

---

## 路线图建议

### Phase 1 — 可靠性（1-2 周）

```
├── #1  graceful shutdown (signal handler + flush)
├── #5  send_with_retry 区分 4xx/5xx
├── #6  session 写入错误反馈
├── #8  LLM transient error 自动重试
└── #11 指数退避
```

### Phase 2 — 安全（1 周）

```
├── #2  Dashboard 加 Bearer token 或 API key auth
├── #3  API key 支持 keyring/system keychain
└── #10 MCP 子进程健康检查
```

### Phase 3 — 性能（1-2 周）

```
├── #13 ui.rs 拆分 + 虚拟滚动
├── #14 Arc<Value> 减少 clone
├── #12 复用 reqwest::Client
└── #20 main_loop 用 tokio::select!
```

### Phase 4 — 测试（持续）

```
├── #19 端到端集成测试
├── edge case 测试（损坏文件、空 config、超长消息）
└── MCP 连接 mock 测试
```

---

## 结论

当前架构已经可以日常使用。P0 里的 **graceful shutdown** 和 **错误重试区分** 是唯一"用了会出问题"的项。其余是生产加固。如果日常使用没遇到明显痛点，可以按 Phase 顺序慢慢推进。
