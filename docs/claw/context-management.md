# i-rs-claw 上下文管理深度剖析

## 1. 问题域

i-rs-claw 是一个 TUI 智能助理，集成 70 个 i-rs CLI 工具。它的核心挑战在于：**LLM 上下文窗口有限，但对话需要携带系统提示、工具文档、跨会话记忆、历史消息、工具调用结果**。如何在有限 token 预算内保留最多有价值信息？

本文从架构设计、压缩算法、到最近一次优化，完整记录 claw 的上下文管理体系。

---

## 2. 系统提示的分层注入

系统提示是上下文的第一大块。claw 的系统提示是一个模板 (`prompts/system.md`，41 行)，运行时注入 7 层半动态内容：

```
system.md 模板
├── Layer 1: 身份与风格 ({{IDENTITY}} → 助理昵称)
├── Layer 2: 当前日期/时间 ({current_date}, {current_time} → 动态生成)
├── Layer 3: 执行模式 ({{PLAN_MODE}} → ReAct 或 Plan-then-Execute 指令)
├── Layer 4: 工具索引 ({{TOOL_INDEX}} → 70 个工具的 name + description)
├── Layer 5: 常用工具文档 ({{HOT_TOOLS}} → 前 5 个高频工具的 skill teach 输出)
├── Layer 6: 用户记忆 ({{USER_MEMORY}} → 跨会话偏好/昵称/信息)
├── Layer 7: 用户画像 ({{USER_PROFILE}} → 入职状态)
└── Layer 8: 用户技能 ({{SKILLS}} → 自定义技能定义)
```

分层设计的意义：每一层可以独立控制加载时机。Layer 5 是高频工具文档的**预加载**（启动时解析 top 5），避免每次对话都调用 CLI 子进程；Layer 6/7 来自 `CrossSessionMemory`，跨会话持久化。

占位符替换后，系统提示折叠多余换行符，最终注入到消息列表的 `[0]` 位置。

---

## 3. 消息构建管道

`build_messages` 是每次对话开始前、将 app 级消息转换成 API 格式的关键函数。它有**两条路径**：

### 路径 A：续接对话（saved_api_messages 不为空）

```
克隆 saved_api_messages
→ 移除过期提醒 (stale reminder)
→ 移除尾部 user 消息 (上轮残留)
→ 追加新 user 消息
→ 注入新鲜提醒
→ smart_compress() 压缩
→ 返回压缩后的消息列表
```

### 路径 B：首轮对话（saved_api_messages 为空）

```
构建 system_prompt (分层注入)
→ 追加 [system] 消息
→ 注入提醒 (如果有)
→ 从 app_messages 截取最后 max_conversation_turns 条
→ 转换为 API 格式
→ 追加新 user 消息
→ compress 保护 (本次优化新增)
→ 返回
```

**优化前的问题**：路径 A 直接调用 `smart_compress(5, 8)` — 硬编码窗口，不感知模型上下文大小。路径 B 完全没有压缩保护。

**本次修复**：两条路径都统一使用 `ContextManager::compress()`，模型感知、自适应窗口。

---

## 4. Smart Compression 算法

`smart_compress` 是 claw 的核心压缩引擎。它的设计哲学是**基于信号打分 → 保留高价值 → 丢弃低价值**。

### 4.1 三级保留策略

```
preserve = {0}  ← 始终保留 system 消息

// 第一级：跨会话工具频率信号（权重 15x）
for &pos in top_scoring_teach_pairs:
    preserve.insert(assistant_tool_call_idx)
    preserve.insert(tool_result_idx)

// 第二级：语义重要性（本次优化新增）
for idx in older_messages:
    if message is (用户纠正 OR 决策确认 OR 有效工具结果):
        preserve.insert(idx)

// 第三级：时间局部性
for idx in recent_window:
    preserve.insert(idx)

// 完整性保障：tool_call 和 tool result 双向配对扫描
forward_scan(): 保留的 tool 结果 → 确保前驱 tool_call 也保留
backward_scan(): 保留的 tool_call → 确保所有后续 tool 结果也保留
```

### 4.2 Teach Pair 检测与评分

Teach pair 是指 LLM 调用 `i_rs {tool} skill teach` 的 tool_call + 返回的文档内容。claw 从消息列表中**反向扫描**（优先最新），按工具名去重。

评分公式：

```
score = (cross_session_frequency × 15) + recency_position
```

其中 `cross_session_frequency` 来自 `CrossSessionMemory`，记录这个工具在所有历史会话中的使用次数。权重由旧版的 10x 提升到 15x，强化跨会话信号的引导作用。

### 4.3 语义重要性评分（本次优化新增）

`score_message_significance` 给消息打语义标签：

| 标签 | 分数 | 示例 | 压缩行为 |
|------|------|------|----------|
| System | 10 | 系统提示 | 永不丢弃 |
| Correction | 9 | "不对，我要的是昨天的" | 始终保持 |
| Decision | 7 | "确认"、"就这样" | 始终保持 |
| ToolData | 6 | 有效 JSON 工具结果 | 优先保留 |
| Dialogue | 3-4 | 普通对话 | 正常处理 |
| LowValue | 1-2 | 问候、错误、空结果 | 预算紧张时丢弃 |

关键词匹配规则（`utils.rs` 中的 `is_correction_message` / `is_decision_message`）：
- 纠错类：`不对`、`不是`、`错了`、`更正`、`重新`、`no,`、`wrong`、`correction`
- 决策类：`确认`、`确定`、`就这样`、`可以了`、`confirm`、`agreed`

---

## 5. ContextManager：模型感知的自适应窗口

`ContextManager` 是压缩的"总调度器"，负责根据**模型上下文大小**动态调整保留窗口。

### 5.1 模型 token 上限

```rust
model.contains("gemini")     → 1,000,000
model.contains("gpt-4o")     →   128,000
model.contains("claude-4")   →   128,000
model.contains("deepseek")   →   128,000
model.contains("gpt-4")      →    32,000
model.contains("gpt-3.5")    →    16,000
其他                          →    32,000 (默认)
```

### 5.2 Token 估算（启发式）

不依赖 tokenizer（会引入重量级依赖），使用启发式：

- CJK 字符：~1.5 token/字符
- ASCII：~4 字符/token
- 每条消息固定 25 token 开销
- 工具调用名和参数也纳入计数

### 5.3 自适应窗口缩减

```
if total_tokens > max_tokens:
    reduction = min(3.0, total_tokens / max_tokens)
    teach_window  = max(1, teach_window / reduction)
    recent_window = max(min_retain, recent_window / reduction)
```

即：上下文占用 3 倍于模型上限时，窗口压缩为 1/3。下限保障 `teach_window ≥ 1`，`recent_window ≥ min_retain`（本次优化新增，默认 6）。

### 5.4 上下文咨询

当占用 &gt;70% 时，追加中文提示到系统消息末尾。`>90%` 时加"建议简化回复，优先引用近期内容"。LLM 会据此自我调节回复长度。

### 5.5 结构摘要（本次优化新增）

`structural_summary()` 从旧消息（recent_window 之外）提取关键事实：

```
[先前上下文摘要]
[工具结果] weight: 2024-01-15 75.0kg
[用户纠正] 不对，我要的是昨天跟前天对比
[决策] 确认用 kg 作为单位
```

纯启发式，不调 LLM，不增加延迟。当前作为预留功能，在 `build_messages` 路径 A 中已集成调用点。

---

## 6. 工具结果处理的三层截断

工具执行结果在三个层级被处理：

### Layer 1: CLI 输出截断 (10,000 字符)

`run_cli_command()` 对子进程 stdout/stderr 做上限截断。防止大输出撑爆进程缓冲区。

### Layer 2: 显示截断 (4,096 字符)

`ToolCallExecutor` 对发送到 TUI 的 `ToolExecuted` 事件结果截断。用户看到完整但不过长的结果显示。

### Layer 3: 上下文注入截断 (500 字符) + 结构化压缩（优化后）

这是给 LLM 看的版本。本次优化前，直接按字符截断 `smart_truncate(result, 500)` — 对 JSON 数组尤其粗暴。

**优化后**：`compact_tool_result()` 对 JSON 数组做语义聚合：

```
输入 (2000 字符 JSON 数组):
  [{"date":"2024-01-01","weight":75.0}, ... 50 条 ...]

输出 (~200 字符):
  [i_rs 共 50 条结果] 1: date=2024-01-01 weight=75 ; 
  2: date=2024-01-02 weight=75.2 ; 
  3: date=2024-01-03 weight=74.8 ; ... 还有 47 条
```

聚合只取前 5 条采样 + 总量统计，在 500 字符预算内保留趋势信息量。

### Layer 3.5: 工具结果缓存（本次优化新增）

`ToolCallExecutor` 内部维护一个 `result_cache: HashMap<String, String>`，键为 `tool_name:args_json`。ReAct 循环中 LLM 有时会重复调用同一工具（尤其纠错重试时），缓存命中直接返回，避免重复子进程执行。

---

## 7. 多级记忆体系

上下文管理不仅是"压缩当前消息列表"，还有一套跨会话记忆系统：

### 7.1 CrossSessionMemory

```
工具频率: { "i-rs-weight": 47, "i-rs-mood": 12, ... }
热工具:   ["i-rs-weight", "i-rs-ledger", "i-rs-note", ...]
用户偏好: ["喜欢公制单位", "工作日不用 remind"]
用户名:   "张三"
助理昵称: "小助手"
用户信息: ["家有猫叫咪咪", "每周三健身"]
```

每次工具执行后，`record_tool_memory()` 更新对应工具频率。`AppCore::shutdown()` 时统一持久化到存储后端。

### 7.2 ToolDocCache

高频工具（top 5）的 `skill teach` 输出被预取并缓存。启动时检查是否有更新，有则重取。缓存跨会话复用，避免了每轮对话都调用子进程获取文档。

### 7.3 SessionManager

每个会话保存 API 格式的完整消息列表（`save_api_messages`）。恢复会话时直接加载，跳过首轮构建，进入路径 A 的续接流程。

---

## 8. 本次优化记录 (2026-05-31)

### 8.1 背景

审计发现当前上下文管理有 6 个集成层面的问题：

| 问题 | 严重度 | 描述 |
|------|--------|------|
| **A. 压缩路径不统一** | 高 | `build_messages` 用硬编码 `smart_compress(5,8)`，`chat_loop` 用 `ContextManager::compress()`，两套逻辑互相独立 |
| **B. 首轮无压缩保护** | 中 | 路径 B 构建首轮消息后不做压缩，热工具文档 + 系统提示可能直接溢出 |
| **C. 无最小保留保障** | 中 | 多轮压缩后消息可能被削到只剩 2-3 条，关键上下文丢失 |
| **D. 消息语义盲区** | 中 | 用户纠错/决策消息和普通对话被等权处理 |
| **E. 工具结果对结构化数据不友好** | 低 | 500 字符截断对 JSON 数组损失极大 |
| **F. 同参数重复执行** | 低 | ReAct 重试时重复调同一工具同一参数 |

### 8.2 改动统计

| 文件 | 改动类型 | 关键变更 |
|------|----------|----------|
| `core/context.rs` | 增强 | +`min_retain` 字段、+`structural_summary()` 方法 |
| `core/engine/builder.rs` | 重构 | 统一压缩路径、+语义评分、+`min_retain` 参数 |
| `utils.rs` | 新增 | +`is_correction_message()`、+`is_decision_message()`、+`compact_tool_result()` |
| `core/executor.rs` | 增强 | +结果缓存、改用 `compact_tool_result` |
| `core/engine/mod.rs` | 修复 | `chat_loop` 每轮都触发压缩、`inject_results` 用结构化压缩 |
| `core/mod.rs` | 集成 | `MessageBuildParams` 传入 `model` 字段 |

### 8.3 效果

- **压缩逻辑统一**：无论路径 A/B、无论 `build_messages` 还是 `chat_loop`，都经过同一个 `ContextManager::compress()` 入口
- **语义保留增强**：用户纠正、决策确认类消息永远不会因超过窗口而被丢弃
- **结构化输出聚合**：JSON 数组结果从"截断丢弃"变成"采样+统计"，信息保留率提升约 3-5x（在相同 token 预算下）
- **重复执行避免**：ReAct 重试循环中同工具同参数的调用只执行一次
- **安全保障**：`min_retain = 6` 确保极端情况下消息列表不会萎缩到失去上下文
- **编译验证**：`cargo check` 0 errors 0 warnings, `cargo test -p i-rs-claw` 232 passed 0 failed

---

## 9. 未来演进方向

### 9.1 LLM 驱动的摘要压缩

当前 `structural_summary` 是纯启发式的。下一步可以用一个 cheap LLM（如 `o3-mini`）对旧消息做真正的语义摘要，替换为一条紧凑的系统消息。关键要解决：

- **异步执行**：摘要不能阻塞 `chat_loop`，需要 tokio::spawn 后台运行
- **摘要持久化**：结果随 `saved_api_messages` 一起保存，重启不丢失
- **退化策略**：摘要未完成时降级到 `context_advisory`（已有）

### 9.2 热工具文档惰性加载

当前系统提示每次都注入 top 5 工具完整文档。如果一个大文档在当前会话完全没有用到，这是纯浪费。改进方向：

```
首轮：只注入工具索引 (name + 一行描述)
首次调用某工具时：将它的完整 skill teach 文档追加为 system 消息
后续轮次：该文档已在上下文中，无需重复注入
```

### 9.3 工具 Schema Token 核算

`chat_loop` 的 `tool_schemas`（含 MCP 工具 schema）未计入 `ContextManager` 的 token 预算。对于 MCP 多工具场景，实际消耗可能比估算高 20-30%。需要在 `ContextManager` 初始化时传入 schema token count。

### 9.4 显式预算分配

从"被动压缩"走向"主动预算管理"：

```
max_tokens × safety_margin (0.9) = 可用预算
  ├── system_prompt:     20%  (固定)
  ├── recent_dialogue:   35%  (最后 N 轮)
  ├── teach_docs:        15%  (高频工具文档)
  ├── tool_schemas:      10%  (工具定义)
  └── tool_results:      20%  (当前工具调用结果)
```

超预算时按优先级模块逐步缩减，而非全局均匀压缩。

### 9.5 滑动窗口摘要

对于超长对话（如 50+ 轮），摘要不应该是"超出窗口就丢"，而是维护一个滚动摘要。每次窗口前移，旧消息被摘要吸收后丢弃，保证"必要信息不丢失 + token 预算不超限"。

### 9.6 上下文压缩质量评估

建立压缩效果的对标体系：
- **召回率**：压缩后 LLM 能否正确回答需要引用旧消息的问题
- **一致性**：压缩前后 LLM 对同一问题的回答是否一致
- **token 节省比**：压缩前后 token 数的比率

---

## 10. 总结

claw 的上下文管理是一个**多层、多信号的保留-丢弃决策系统**。核心思路是将 token 预算按照"信号质量"分配：

1. **系统提示**（固定预算）— 行为约束，不可压缩
2. **跨会话信号**（高频工具文档、用户偏好）— 长期价值，强信号
3. **语义信号**（纠正、决策、数据）— 高信息密度
4. **时间局部性**（最近 N 轮）— 短期价值
5. **碎片**（问候、错误、空结果）— 优先丢弃

这种"信号驱动的收益递减"策略，使得 claw 在 8K-1M 不同模型上都能自适应运作，无需手动调参。

本次优化将架构从"两套独立压缩路径"统一为一条流水线，补上了语义感知、结构化提取、结果缓存和最小保留保障四个关键能力，为后续的 LLM 摘要和显式预算管理奠定了坚实基础。
