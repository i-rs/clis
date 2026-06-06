# i-rs-claw 系统提示词 Token 消耗分析报告 (v3)

**分析日期:** 2026-06-06  
**修订:** v3 — 修正 reasoning/SKILLS/TOOL_INDEX 分析偏差, 聚焦可执行优化

---

## 1. 执行摘要

系统提示词构建存在 **5 个可优化点**，其中 1 个是 bug（仅影响 `deepseek-reasoner`）。

| 用户场景 | 当前 Token | 优化后 Token | 节省 |
|----------|----------|-------------|------|
| 新用户 (0 工具) | ~2,000 | ~1,800 | ~10% |
| 典型用户 (10 工具, 3 热工具) | ~7,000 | ~2,500 | ~64% |
| 重度用户 (30 工具, 5 热工具, 5 技能) | ~18,000 | ~3,500 | ~80% |

---

## 2. 修正: 3 个先前分析偏差

### 2.1 `reasoning_content` 回传
- ❌ 原分析: 对所有模型都是浪费
- ✅ 修正:
  - `deepseek-chat` **不产出** `reasoning_content` → sse.rs:220-225 永不触发 → 不影响
  - `deepseek-reasoner` **产出并必须剥离** → DeepSeek 官方: "若输入含 reasoning_content 返回 400 错误" → **多轮对话 bug**
  - GPT-4o / Claude 不产出 → 不影响

### 2.2 `{{SKILLS}}` 注入
- ❌ 原分析: 75 个预置 SKILL.md 全量注入 → 35K tokens
- ✅ 修正: 仅用户通过 `skill install` **手动安装**的自定义技能。典型用户 **0 个技能 → 0 tokens**。存储路径 `~/.i-rs/claw/agents/{agent_id}/skills/`

### 2.3 `{{TOOL_INDEX}}` 工具数
- ❌ 原分析: 全部 70 个工具注入
- ✅ 修正: 仅 `config.i_rs_tools` 中**已配置且二进制已安装**的工具。典型用户 5-20 个。`discover_i_rs_tools()` 对每个工具运行 `i-rs-{name} skill summary`，失败则跳过。

---

## 3. 5 个可优化项

### 3.1 🔴 [BUG] `reasoning_content` 回传 → `deepseek-reasoner` 多轮 400 错误

**位置:** `src/core/engine/mod.rs:168,389` / `src/providers/sse.rs:143`

**现象:** `deepseek-reasoner` 流式返回 `reasoning_content` → 存储到 assistant 消息 JSON → 下一轮 API 调用原样发送 → **400 Bad Request**

**修复:** 在 `sse.rs` 发送 `messages` 到 API 前，迭代每条消息 remove `"reasoning_content"` 键。

```rust
// 在 openai_stream_chat_impl 中, 发送 request body 前:
let cleaned: Vec<Value> = messages.iter().map(|m| {
    let mut m = m.clone();
    m.as_object_mut().map(|o| o.remove("reasoning_content"));
    m
}).collect();
body["messages"] = Value::Array(cleaned);
```

**难度:** 🟢 低 (约 5 行代码)  
**仅影响:** `deepseek-reasoner` 用户  
**不影响的模型:** `deepseek-chat`, GPT-4o, Claude, 所有其他

---

### 3.2 🟡 [优化] HOT_TOOLS 注入完整 `skill teach` 文档

**位置:** `src/tool_cache.rs:63-83` (format_hot_tools)  
**调用链:** `core/mod.rs:450-453` → `builder.rs:85`

**现象:** 常用工具 (最多 5 个) 的完整 CLI 教学文档注入系统提示词。每个 teach 文档 200-1,000+ 字符。

```
## 常用工具文档
以下是你最近常用的工具的完整教学文档：

### weight
i-rs-weight 是用于管理体重记录的CLI工具。
支持的命令:
  add     - 添加体重记录
  list    - 列出体重记录...
  ... (200-1000+ 行)
```

**为什么是浪费:** 模型已有 `i_rs` 工具可以按需调用 `command=skill args=["teach"]` 获取相同文档。注入静态副本不会提高正确性（模型仍需查询参数格式），但增加 ~2,500 tokens/请求。

**优化方案 A (激进):** 移除 `{{HOT_TOOLS}}` 占位符，完全依赖按需 `skill teach`。  
**优化方案 B (保守):** `format_hot_tools` 仅输出工具名+摘要 (1 行/工具，约 20 tokens)：
```rust
result.push_str(&format!("- {} ({}次使用)\n", tool, freq));
```

**难度:** 🟢 低  
**节省:** 0-2,500 tokens/请求 (有热工具时)

---

### 3.3 🟡 [优化] `i_rs` 工具参数描述 4 次重复 "teach-first"

**位置:** `src/tools/i_rs.rs:17,27,31,36`

**当前描述字符数:** ~600 chars (~150 tokens)

```
description: "执行 i-rs CLI 命令...使用标准流程：先用 command=skill args=[\"teach\"] 学习..."

tool.description: "首次使用不熟悉的工具时，先调用 command=skill args=[\"teach\"] 获取完整命令文档再操作。"

command.description: "必须先用 skill teach 确认工具支持哪些命令（add/list/get/delete/update/stats 等），不要猜测。skill 子命令用法：command=\"skill\", args=[\"teach\"]"

args.description: "严格按 skill teach 返回的文档中的参数顺序传入..."
```

**优化后 (~280 chars, ~70 tokens):**
```rust
description: "执行 i-rs CLI 管理用户数据。不熟悉的工具先用 command=skill args=[\"teach\"] 学习。"

tool.description: "工具名称"
command.description: "子命令 (add/list/get/delete/update/stats/skill 等)"
args.description: "参数数组，日期用 YYYY-MM-DD 格式"
// explanation 保持不变
```

**难度:** 🟢 低  
**节省:** ~80 tokens/请求 (永久)

---

### 3.4 🟡 [优化] `{{TOOL_INDEX}}` 与 API Tool Schema 内容重叠

**位置:** `src/core/mod.rs:637-661` (build_irs_tool_index)  
**调用链:** `core/mod.rs:416` → `builder.rs:83`

**现象:** TOOL_INDEX 在市场提示词中输出:
```
## i-rs 工具索引
- weight: 体重记录管理 (add/list/get/delete/update/stats/chart)
- mood: 情绪记录管理 (add/list/get/delete/update/stats/calendar)
...
```

同时 API `tools` 数组中 `i_rs` 工具的 `tool` 参数也有 `enum: ["weight", "mood", ...]` 列出全部可用工具。

**分析:** 工具名称列表两边都有，但 TOOL_INDEX 额外包含每个工具的**描述文本** (来自 `skill summary`)。这是 API schema 中没有的信息。

**优化方案 A:** 保留 TOOL_INDEX 但仅输出**工具名列表** (无描述)，约 30 tokens (vs 当前 ~300-800 tokens)。  
**优化方案 B:** 移除 TOOL_INDEX，将所有工具描述放到 `i_rs` tool 的 `tool.enum` 中作为 labeled values（但 JSON Schema 不支持）。

**推荐方案 A:** 压缩为纯工具名列表。模型可以从 API schema 的 enum 知道有哪些工具，从 TOOL_INDEX 的一行一工具名快速扫描可用范围。

**难度:** 🟢 低  
**节省:** ~200-500 tokens/请求

---

### 3.5 🟢 [优化] 对话历史: 降低压缩阈值

**位置:** `src/core/engine/builder.rs:527-528` / `src/core/context.rs:86,132`

**当前设置:**
- `recent_keep=20` — 最近 20 条消息不压缩
- `max_tokens=128,000` (deepseek/gpt-4o/claude) — 超过此阈值才压缩
- 工具密集型对话: 20 条消息可包含大量 tool_call/tool_result 对 (每对 ~200-500 tokens)

**优化:**
```rust
// builder.rs:528
smart_compress(msgs, tool_frequency, 5, 12, 6);
//                                   ↑  ↑
//                         teach_docs  recent_keep (20 → 12)
```

```rust
// context.rs:132 — 添加主动压缩
if total > self.max_tokens / 2 {  // 超过 50% 预算时轻度压缩
    smart_compress(msgs, tool_frequency, 3, 8, 6);
}
```

**难度:** 🟢 低  
**节省:** 渐进式，长对话场景 ~20-30%

---

### 3.6 🟢 [次要] 其他低优先级项

| 项目 | 说明 | 优先级 |
|------|------|--------|
| user_info/preferences 截断 | `memory.rs:209,218` 加 `.truncate(10)` | 🟢 低 |
| 子代理不复制 TOOL_INDEX | `delegate.rs:334-351` 代理任务不需要 i-rs 列表 | 🟢 低 |
| 新用户引导文本 | `memory.rs:296-304` ~80 tokens 重复直到用户填写 profile | 🟢 低 |
| MCP 工具过滤 | 仅启用的 MCP server 才加载工具 schema | 🟢 低 |

---

## 4. 修复建议优先级

| 优先级 | 项目 | 类型 | 难度 | 节省 Token |
|--------|------|------|------|-----------|
| **P0** | 3.1 reasoning_content 剥离 | **BUG** | 5行 | 1-2K/轮 |
| **P1** | 3.3 压缩 i_rs 参数描述 | 优化 | 10行 | ~80/请求 |
| **P1** | 3.2 压缩 HOT_TOOLS | 优化 | 5行 | 0-2.5K/请求 |
| **P2** | 3.4 精简 TOOL_INDEX | 优化 | 3行 | ~400/请求 |
| **P2** | 3.5 降低压缩阈值 | 优化 | 2行 | ~20-30%/长对话 |
| **P3** | 3.6 次要项 | 优化 | — | 渐进 |

---

## 5. 优化后预期效果

| 组件 | 优化前 | 优化后 | 变化 |
|------|--------|--------|------|
| Base 模板 | ~450 | ~450 | — |
| i_rs 参数描述 | ~150 | ~70 | -53% |
| TOOL_INDEX (10 工具) | ~400 | ~60 | -85% |
| HOT_TOOLS (3 工具) | ~1,500 | ~60 | -96% |
| {{SKILLS}} (典型 0) | 0 | 0 | — |
| API tool schemas | ~500 | ~500 | — |
| 对话历史 | ~2,000 | ~1,500 | -25% |
| **合计 (典型用户)** | **~7,000** | **~2,600** | **-63%** |
