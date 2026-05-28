# i-rs-code 成熟度提升计划

> 基于全面审计，从 P0（阻断性）到 P3（优化）共发现 24 项问题。本计划按优先级分 4 个阶段执行。

---

## Phase 0: 基础保障 (预计 1-2 天)

> 目标：消除 crash 风险，建立 CI 安全网

### P0-1: CI 覆盖 i-rs-code
- `check.yml` 添加 `cargo test -p i-rs-code -- --test-threads=1` job
- 添加 clippy + fmt 检查（已通过 workspace 级别覆盖，确认无误即可）
- **验收**: PR 触发 CI 绿色通过

### P0-2: 消除 unwrap() — 分 crate 批量替换
- `cli-api/src/main.rs` (71 个) → 优先级最高，逐个审查替换为 `?` 或 `expect("上下文")`
- `mcp/tests/integration_test.rs` (27 个) → 测试中可保留部分 `unwrap`，加注释说明
- `claw/src/tools/file_ops.rs` (25 个) → 审查后替换
- 其余 86 个文件中的 unwrap → 按文件逐个处理
- **规则**: 测试代码中 `unwrap()` 可保留但加 `// ok: test context`；生产代码一律消除
- **验收**: `grep -r '\.unwrap()' crates/code/src/` 结果为空或仅有测试代码

### P0-3: panic! 消除
- `claw/src/tools/file_ops.rs` (4 个) → 替换为 `anyhow::bail!`
- `claw/src/dashboard/routes.rs` (3 个) → 替换为返回 `ApiError`
- `code/src/session.rs` (1 个) → 替换
- **验收**: 生产代码中无 `panic!` 调用

### P0-4: unsafe 代码审查
- `claw/src/config.rs` (5 个) → 逐个审查是否有 safe 替代
- 其余 3 处 → 审查注释 justify
- **验收**: 每个 unsafe 块都有 `// SAFETY: ...` 注释

---

## Phase 1: 安全 + 工具质量 (预计 3-5 天)

> 目标：沙箱可靠，工具输出可控

### P1-1: Bash 沙箱重写
- **白名单模式替代黑名单**: 定义允许的命令前缀（`cargo`, `git`, `ls`, `cat`, `grep`, `python`, `node`, `npm`, `go`, `rustc` 等）
- **命令超时**: 添加默认 120s 超时（`tokio::time::timeout`），可在 args 中配置
- **输出截断**: stdout/stderr 各限制 64KB，超出截断并标注 `[truncated N bytes]`
- **禁止交互式命令**: 屏蔽 `vim`/`nano`/`less`/`ssh` 等需要 TTY 的命令
- **工作目录验证**: `cd` 命令后验证仍在 workspace 内
- **验收**: 安全测试套件通过 + 绕过尝试全部被拦截

### P1-2: 工具输出截断保护
- `ToolResult` 添加最大输出限制（默认 32KB）
- 超出时截断并附加 `[output truncated, original size: N]`
- 在 `react_loop` 中对 tool result 做预检查，超限时自动摘要
- **验收**: 大文件 cat 不再撑爆上下文

### P1-3: 上下文压缩优化
- **压缩时用 LLM 生成摘要**: 超限时调用 provider 生成对话摘要（非简单截断）
- **保留关键信息**: 修改的文件路径、错误消息、最终结果不截断
- **工具结果智能截断**: 保留头部和尾部，中间省略（而非仅保留 200 字符头部）
- **验收**: 压缩后的上下文仍然包含足够信息让 LLM 继续任务

### P1-4: 结构化错误类型
```rust
pub enum ToolError {
    NotFound { path: String },
    PermissionDenied { path: String },
    Timeout { tool: String, seconds: u64 },
    InvalidArgs { field: String, reason: String },
    ExternalError { tool: String, message: String },
}
```
- 工具返回结构化 JSON 错误而非纯字符串
- LLM 能区分不同错误类型并采取不同策略
- **验收**: 所有 15 个工具返回结构化错误

---

## Phase 2: 工具增强 (预计 5-7 天)

> 目标：补齐关键工具，达到可用水平

### P2-1: LSP 增强
- 添加语言映射: C/C++ (`clangd`), Java (`jdtls`), Kotlin, Swift (`sourcekit-lsp`), Dart (`dart analyze`)
- 添加 `lsp_rename` 工具（`textDocument/rename`）
- 添加 `lsp_hover` 工具（获取类型信息）
- 添加 `lsp_code_action` 工具（自动修复建议）
- **LSP 服务器不存在时优雅降级**：返回 "LSP not available for this language" 而非 crash
- **验收**: rename/hover 在 Rust 项目中正常工作

### P2-2: Git 深度集成
- `git` 工具增加: `stash`/`stash_pop`/`checkout_file`/`diff_staged`
- 与 undo 集成: 文件修改前自动 `git stash`
- 添加 `git_status` 结构化输出（不只是文本）
- **验收**: 文件修改可通过 git 一键回滚

### P2-3: Project 感知
- 启动时自动检测项目类型（Rust/Node/Python/Go 等）
- 读取项目配置（`Cargo.toml`/`package.json`/`pyproject.toml`）注入 system prompt
- 支持 `AGENTS.md` / `.cursor/rules` / `.github/copilot-instructions.md` 自动注入
- 支持用户自定义 `~/.i-rs-code/instructions.md`
- **验收**: 打开 i-rs 项目时，AGENTS.md 内容自动注入 system prompt

### P2-4: System Prompt 增强
- 支持分层 prompt: base + project + user
- 工具描述动态注入（含当前项目的实际用法示例）
- 支持中文/英文双语 prompt
- **验收**: 不同项目打开时 system prompt 不同

### P2-5: 渐进式验证链
- 添加 `verify` 工具: 按顺序执行 check → clippy → test → fmt
- 每步失败时自动读取错误并提示 LLM 修复
- 支持项目特定验证配置
- **验收**: 代码修改后自动验证链工作正常

### P2-6: 多文件编辑原子性
- 添加 `batch_edit` 工具: 接受多个文件编辑操作，全部成功或全部回滚
- 内部实现: 先 apply 到内存，全部成功后写入磁盘
- 失败时通过 git stash 自动回滚
- **验收**: 3 文件同时编辑，中间一个失败时全部回滚

---

## Phase 3: 工程质量 (预计 3-5 天)

> 目标：文档、可观测性、成本控制

### P3-1: 文档覆盖
- `pub trait Tool` 添加完整 doc comment + 使用示例
- `Agent` 结构体添加模块级 `//!` 文档
- 所有 `pub fn` 添加 `///` doc（目标 >60% 覆盖率）
- 添加 `examples/` 目录含基础使用示例
- **验收**: `cargo doc --no-deps` 无警告

### P3-2: 结构化日志 + 可观测性
- 引入 `tracing` crate 替代 `println!`
- 工具调用记录: tool_name, args_hash, duration_ms, result_status
- 添加 `tracing-subscriber` 支持 JSON 日志输出
- TUI debug panel 消费 tracing 事件
- **验收**: 工具调用有完整审计日志

### P3-3: 成本控制
- 添加 token 预算配置: `max_input_tokens`, `max_output_tokens_per_call`
- 工具输出自动截断（per-tool 限制）
- 会话结束时显示消耗统计: tokens + 估算费用
- 超预算时提前终止并提示用户
- **验收**: 超预算会话自动停止并报告消耗

### P3-4: 性能优化
- `ToolRegistry::get()` 返回引用而非 clone Arc
- `build_messages()` 使用 `Cow<str>` 避免不必要的 String clone
- `LlmMessage` 考虑用 `Arc<str>` 共享大文本
- **验收**: 大上下文（100+ 消息）场景下内存使用降低 30%+

### P3-5: 测试增强
- 添加集成测试: 端到端 "修改文件 → cargo check → 修复" 场景
- 添加 bash 沙箱安全测试: 30+ 绕过尝试
- 添加 LSP mock 测试（不依赖 rust-analyzer）
- 添加 context compression 正确性测试
- **验收**: 测试数量翻倍，覆盖关键路径

---

## 执行优先级总结

| 阶段 | 工作量 | 阻断程度 | 建议顺序 |
|------|--------|---------|---------|
| Phase 0: CI + unwrap | 1-2 天 | 阻断性 | **立即** |
| Phase 1: 安全沙箱 | 3-5 天 | 严重影响 | Phase 0 完成后 |
| Phase 2: 工具增强 | 5-7 天 | 功能缺失 | Phase 1 完成后 |
| Phase 3: 工程质量 | 3-5 天 | 长期健康 | 可与 Phase 2 并行 |

**总工作量估算**: 12-19 天（含测试 + 验收）
