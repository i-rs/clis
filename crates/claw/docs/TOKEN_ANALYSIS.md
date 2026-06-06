# i-rs-claw 系统提示词 Token 消耗分析报告

**分析日期:** 2026-06-06  
**分析范围:** `crates/claw/src/core/engine/builder.rs`, `context.rs`, `tools/`, `skill_store.rs`, `memory.rs`, `tool_cache.rs`, `providers/`

---

## 1. 执行摘要

i-rs-claw 的系统提示词构建存在显著的 token 浪费。**单次 API 调用的系统提示词可达 40,000+ tokens**，其中 55-65% 为冗余或可优化内容。按 GPT-4o 定价 ($2.50/1M input)，每次对话回合浪费约 $0.05-0.10，跨数千次对话和子代理委托后累计显著。

**核心问题:** 大量静态内容被无条件注入每次 API 调用，且从未被模型实际使用。

---

## 2. Token 消耗全景图

| # | 组件 | 条件 | 典型 Token | 最大 Token | 浪费评级 |
|---|------|------|-----------|-----------|---------|
| 1 | Base 模板 (system.md) | Always | ~450 | 450 | 🟢 低 |
| 2 | 日期/时间占位符 | Always | ~10 | 10 | 🟢 无 |
| 3 | Plan mode prompt | Always | ~40-120 | 120 | 🟢 低 |
| 4 | **TOOL_INDEX** (i-rs 工具目录) | Always | ~2,600 | ~3,500 | 🔴 严重 |
| 5 | **HOT_TOOLS** (完整 teach 文档) | Conditional | 0-2,500 | 3,000+ | 🔴 严重 |
| 6 | **SKILLS** (全部技能文件) | Always (if exist) | **~35,000** | **50,000+** | 🔴 严重 |
| 7 | USER_MEMORY | Conditional | 0-300 | 500+ | 🟡 中等 |
| 8 | USER_PROFILE | Always | 50-200 | 200 | 🟢 低 |
| 9 | ROUTING_HINT | Conditional | 0-200 | 200 | 🟢 低 |
| 10 | Reminder injection | Conditional | 0-200 | 200 | 🟢 低 |
| 11 | **API tool schemas** | Every call | ~1,500 | 4,000+ | 🟡 中等 |
| 12 | **Conversation history** | Every call | 500-5,000 | 50,000+ | 🟡 中等 |
| 13 | **Reasoning content** | Every turn (DeepSeek) | **1,000-2,000** | **per turn** | 🔴 严重 |

---

## 3. 关键发现

### 3.1 🔴 CRITICAL: 全部技能文件注入系统提示词

**位置:** `src/skill_store.rs:274-326` → `format_skills()`  
**代码路径:** `builder.rs:86` → `("{{SKILLS}}", skills)`

- `skills/` 目录下有 **75 个 SKILL.md 文件**，总计 **~140KB / ~35,000 tokens**
- `format_skills()` 将所有文件的**完整内容**无条件拼接到系统提示词
- 模型已经可以通过 `skill_{name}` 工具按需加载技能文档
- 几乎没有任何一个对话会用到超过 3-5 个技能

**浪费:** 每个请求 35,000 tokens，但 95%+ 的技能内容从未被使用。  
**修复:** 移除 `{{SKILLS}}` 占位符，完全依赖按需 `skill_{name}` 工具加载。

---

### 3.2 🔴 CRITICAL: Reasoning Content 回传 API

**位置:** `src/core/engine/mod.rs:168-169, 389-391` — reasoning_content 存储在消息 JSON  
**位置:** `src/providers/sse.rs:140-143` — messages 原样发送，**未剥离 reasoning_content**

- DeepSeek 等模型产生大量 `reasoning_content`（每次工具调度和文本回复都有）
- 这些 reasoning 链被存储在 assistant 消息的 JSON 中
- **下一轮 API 调用时，所有历史 reasoning 链被重新发送**
- N 轮对话 → (N-1) × ~1,000-2,000 tokens 的回传浪费

**浪费:** 10 轮对话产生 10,000-20,000 个仅用于回传的 reasoning tokens。  
**修复:** 在发送 API 请求前剥离消息中的 `reasoning_content` 字段。注：Anthropic provider 已经在转换时跳过了，但 OpenAI-compatible 路径未处理。

---

### 3.3 🔴 CRITICAL: TOOL_INDEX 与 API Tool Schema 完全重复

**位置:** `src/core/mod.rs:637-661` → `build_irs_tool_index()`  
**代码路径:** `builder.rs:84` → `("{{TOOL_INDEX}}", tool_index)`

- `build_irs_tool_index()` 将 70 个 i-rs CLI 工具格式化为 Markdown 列表  
- 每个工具一行描述，共 ~2,600 tokens
- **这些工具的 `name` + `description` + `parameters` 已经通过 API 的 `tools` 数组发送**
- 模型从 tool schema 中就能知道有哪些工具可用
- 系统提示词中的工具列表是**完全冗余的**

**浪费:** 每个请求 2,600 tokens 的重复信息。  
**修复:** 移除 `{{TOOL_INDEX}}` 占位符；模型已从 API tool schemas 获取工具信息。

---

### 3.4 🔴 CRITICAL: HOT_TOOLS 注入完整 CLI 教学文档

**位置:** `src/tool_cache.rs:63-83` → `format_hot_tools()`  
**代码路径:** `builder.rs:85` → `("{{HOT_TOOLS}}", hot_tools)`

- 对最常用的 5 个工具调用 `i-rs <tool> skill teach` 获取完整文档
- 每个 teach 文档包含命令签名、参数、示例（200-1,000+ 字符）
- 5 个工具 × ~500 tokens/个 = **~2,500 tokens** 每请求
- 模型可以按需调用 `skill teach`

**浪费:** 2,500 tokens 每次对话，且与 TOOL_INDEX 重复。  
**修复:** 仅保留工具名 + 1 行摘要（或完全移除，依赖按需查询）。

---

### 3.5 🟡 HIGH: i_rs 工具参数描述 4 次重复 "teach-first"

**位置:** `src/tools/i_rs.rs:16-17, 27, 31, 36, 40`

- tool 描述: "使用标准流程：先用 command=skill args=["teach"]..."
- `tool` 参数: "首次使用不熟悉的工具时，先调用 command=skill..."
- `command` 参数: "必须先用 skill teach 确认工具支持哪些命令..."
- `args` 参数: "严格按 skill teach 返回的文档中的参数顺序传入..."
- `explanation` 参数: "用中文简要解释当前操作"

**"teach-first" 指令在 4 个字段中重复了 4 次。**

**浪费:** ~200 tokens 的冗余文本。  
**修复:** 将 teach-first 指令压缩到 tool description 中一行，参数描述仅保留功能说明。

---

### 3.6 🟡 HIGH: 用户记忆向量无大小上限

**位置:** `src/memory.rs:29-30, 209, 218`

```rust
user_info: Vec<String>,      // 无上限
preferences: Vec<String>,    // 无上限
```

- `add_user_info` 和 `add_preference` 仅做去重，不做截断
- 每次系统提示词注入所有 user_info 和 preferences
- 随着使用时间增长会无限膨胀

**修复:** 添加 `user_info.truncate(10)` 和 `preferences.truncate(10)` 或基于 token 预算的截断。

---

### 3.7 🟡 MEDIUM: 对话历史保留 20 轮

**位置:** `src/core/engine/builder.rs:527-528`

```rust
smart_compress(msgs, tool_frequency, 5, 20, 6);
// max_teach_docs=5, recent_keep=20, min_retain=6
```

- `recent_keep=20` — 保留最近 20 条消息不压缩
- 对于工具密集型对话，20 条消息可能包含大量 tool_call/tool_result 对
- 自适应 token 预算检查仅在全上下文超限时触发

**修复:** 将 `recent_keep` 从 20 降至 8-10，或基于 token 计数而非消息计数。

---

### 3.8 🟡 MEDIUM: 压缩阈值过高 (128K)

**位置:** `src/core/context.rs:132-156`

```rust
pub fn compress(&mut self, msgs: &mut Vec<Value>, ...) {
    let total = self.token_counter.count_messages(msgs);
    if total <= self.max_tokens {
        return; // 不超过 128K 就不压缩
    }
```

- 仅在超过 `max_tokens`（通常 128K）时才触发压缩
- 在此阈值以下，所有历史消息、reasoning、冗余内容都留在上下文中
- 实际可用上下文空间被大量无效内容消耗

**修复:** 添加主动压缩策略（如超过 50% 预算时轻度压缩），或基于消息年龄的逐出策略。

---

### 3.9 🟢 LOW: 其他次要发现

| 发现 | 位置 | 说明 |
|------|------|------|
| 新用户引导提示词每轮重复 | `memory.rs:296-304` | ~80 tokens 的引导文本，用户填写 profile 后会消失 |
| structural_summary 仅 10 条事实 | `context.rs:209` | 这是好的（限制大小），但旧消息未做 LLM 摘要 |
| 子代理委托复制工具索引 | `delegate.rs:334-351` | 每次 delegate_task 重新注入全部 TOOL_INDEX |
| MCP 工具动态增加 schema | `tools/mcp_tools.rs` | 每个 MCP 工具增加 ~50-200 tokens schema |
| 没有发送前 token 预算检查 | `engine/mod.rs` | 可能在 API 调用失败时才发现超限 |

---

## 4. 修复优先级矩阵

| 优先级 | 问题 | 难度 | 节省 Token/请求 | 影响 |
|--------|------|------|----------------|------|
| **P0** | 移除 `{{SKILLS}}` 全量注入 | 🟢 低 | **35,000** | 所有请求 |
| **P0** | 剥离 API 请求中的 `reasoning_content` | 🟢 低 | **1,000-2,000/轮** | 多轮对话 |
| **P0** | 移除 `{{TOOL_INDEX}}` 或压缩为 1 行 | 🟢 低 | **2,600** | 所有请求 |
| **P1** | 压缩 `{{HOT_TOOLS}}` 为工具名+摘要 | 🟡 中 | **2,000** | 有热工具时 |
| **P1** | 压缩 i_rs 工具参数描述 | 🟢 低 | **200** | 所有请求 |
| **P2** | user_info/preferences 截断 | 🟢 低 | 渐进式节省 | 长期使用 |
| **P2** | 降低 recent_keep 和压缩阈值 | 🟢 低 | 渐进式节省 | 长对话 |
| **P3** | 子代理不复制 TOOL_INDEX | 🟡 中 | 2,600/委托 | 多代理场景 |

**P0 三项修复可节省 ~40,000 tokens/请求（约 90% 的系统提示词体积）。**

---

## 5. 优化后预期 Token 预算

| 组件 | 当前 | 优化后 |
|------|------|--------|
| Base 模板 | ~450 | ~450 |
| TOOL_INDEX | ~2,600 | **0** (移除) |
| HOT_TOOLS | ~2,500 | **200** (压缩) |
| SKILLS | ~35,000 | **0** (按需) |
| USER_MEMORY | ~300 | ~300 (截断) |
| API tool schemas | ~1,500 | ~1,200 (压缩描述) |
| Conversation history | ~2,000 | ~1,500 (更低压缩阈值) |
| **系统提示词合计** | **~44,000** | **~3,500** |
| **节省** | — | **~40,500 tokens (92%)** |
