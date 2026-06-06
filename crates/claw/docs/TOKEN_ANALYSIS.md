# i-rs-claw 系统提示词 Token 消耗分析报告 (v2 修订版)

**分析日期:** 2026-06-06  
**修订:** v2 — 修正 3 个分析偏差

---

## v2 修订说明

| 偏差 | 原分析 | 修正后 |
|------|--------|--------|
| {{SKILLS}} 35K tokens 浪费 | 误认为 75 个预置 SKILL.md 文件全部注入 | **用户手动安装的技能**，典型用户 0 个技能 → 0 tokens |
| reasoning_content 回传浪费 | 标记为可安全剥离 | DeepSeek-reasoner 官方文档要求**必须剥离**（含此字段→400 错误），是 **bug 非仅优化** |
| TOOL_INDEX 70 个工具浪费 | 假设全部 70 个工具都注入 | 仅注入 `config.i_rs_tools` 中**已配置且已安装**的工具，典型 5-20 个 |

---

## 1. 执行摘要

i-rs-claw 的系统提示词构建存在 token 浪费，但严重程度需正确评估。不同用户配置下差异极大：

| 用户场景 | 系统提示词 Token | 浪费 Token | 浪费比例 |
|----------|-----------------|-----------|---------|
| 新用户 (0 工具, 0 技能) | ~2,000 | ~300 | ~15% |
| 典型用户 (10 工具, 0 技能) | ~4,500 | ~2,000 | ~44% |
| 重度用户 (30 工具, 5 技能, 5 热工具) | ~15,000 | ~8,000 | ~53% |

**核心问题:** 系统提示词体积随工具数量**线性增长**，且存在冗余内容。

---

## 2. 修正后的 Token 消耗全景图

| # | 组件 | 条件 | 典型 Token | 浪费评级 | 修正说明 |
|---|------|------|-----------|---------|---------|
| 1 | Base 模板 (system.md) | Always | ~450 | 🟢 低 | — |
| 2 | TOOL_INDEX (i-rs 工具目录) | 有 i_rs_tools 配置 | ~150/tool | 🟡 中等 | **修正:** 仅已配置+已安装的工具 |
| 3 | HOT_TOOLS (完整 teach 文档) | 有热工具时 | ~500/tool | 🔴 严重 | 完整 teach 文档注入每轮 |
| 4 | **SKILLS (用户自定义技能)** | **用户安装后** | **0-10,000+** | 🟡 中等 | **修正:** 非预置 75 文件；是用户手动安装 |
| 5 | `reasoning_content` 回传 | 每轮 (reasoning 模型) | **1,000-2,000/轮** | 🔴 严重 (BUG) | **修正:** DeepSeek 要求必须剥离，否则 400 错误 |
| 6 | API tool schemas | Every call | ~100/tool | 🟡 中等 | — |
| 7 | i_rs 参数描述 | Always | ~200 | 🟡 中等 | teach-first 重复 4 次 |
| 8 | Conversation history | Every call | 500-5,000 | 🟡 中等 | — |

---

## 3. 关键发现 (修正后)

### 3.1 🔴 CRITICAL BUG: `reasoning_content` 回传导致 DeepSeek-reasoner 400 错误

**位置:** `src/core/engine/mod.rs:168-169, 389-391` / `src/providers/sse.rs:140-143`

**DeepSeek 官方文档明文规定:**
> "If the `reasoning_content` field is included in the sequence of input messages, the API will return a `400` error. Therefore, you should remove the `reasoning_content` field from the API response before making the API request."

当前代码在存储 assistant 消息时保留了 `reasoning_content`，并在下一轮 API 调用中原样发送。这意味着：
- **`deepseek-reasoner` 多轮对话必然失败（400 错误）**
- `deepseek-chat` 等标准模型会收到多余的 `reasoning_content` 字段（浪费 tokens 但不会报错）
- Anthropic 路径不受影响（转换时已隐式过滤）

**修复:** 在 `sse.rs` 发送消息前剥离所有消息中的 `reasoning_content` 字段。

---

### 3.2 🔴 CRITICAL: HOT_TOOLS 注入完整 CLI 教学文档

**位置:** `src/tool_cache.rs:63-83` → `format_hot_tools()`

同上版分析，此问题不变。每轮注入 5 个常用工具的完整 `skill teach` 输出（每工具 200-1,000+ 字符）。

**修复:** 仅保留工具名 + 1 行摘要，或完全移除依赖按需 `skill teach` 查询。

---

### 3.3 🟡 HIGH: i_rs 工具参数描述重复 "teach-first"

**位置:** `src/tools/i_rs.rs:16-17, 27, 31, 36, 40`

不变。同一指令在 4 个字段中重复。

**修复:** 压缩到 tool description 中一行。

---

### 3.4 🟡 MEDIUM: TOOL_INDEX 与 API Schema 内容重叠

**位置:** `src/core/mod.rs:637-661` / `builder.rs:84`

修正后的结论：TOOL_INDEX 只包含已配置+已安装的工具（非全部 70 个）。但仍与 API tool schemas 部分重叠 — 工具名称和描述在两边都出现。

**典型影响:** 10 个工具 ≈ 1,500 chars / ~400 tokens。  
**修复:** 考虑将 TOOL_INDEX 替换为更精简的格式（仅工具名列表），或完全移除依赖 schema 中的描述。

---

### 3.5 🟡 MEDIUM: {{SKILLS}} — 用户自定义技能（非 75 预置文件）

**位置:** `src/skill_store.rs:274-326` → `format_skills()`

**修正:** {{SKILLS}} 占位符加载的是用户通过 `i-rs-claw skill install <name>` **手动安装**的自定义 `.md` 文件。这些文件存储在 `~/.i-rs/claw/agents/{agent_id}/skills/`，默认目录为空。

- 典型用户安装 0 个技能 → 占用 0 tokens
- 高级用户可能安装 1-5 个技能 → 占用数百到数千 tokens
- 75 个预置 `skills/i-rs-{name}/SKILL.md` 文件是**按需加载的**（通过 `skill teach` 命令），不会被注入系统提示词

**此问题严重度从 P0 降级。** 但仍建议：用户安装的技能若包含参数（callable as tool），系统提示词应仅保留摘要，模型可通过 `skill_{name}` 工具按需获取完整内容。

---

### 3.6 🟢 LOW: 其他发现（与 v1 一致）

| 发现 | 说明 |
|------|------|
| user_info/preferences 无大小上限 | 长期使用会累积，应加 truncate |
| recent_keep=20 过高 | 建议降至 8-10 |
| 压缩阈值 128K 过高 | 可在 50% 时主动轻度压缩 |
| 子代理委托复制 TOOL_INDEX | delegate 无需 i-rs 工具列表 |

---

## 4. 修正后的修复优先级

| 优先级 | 问题 | 难度 | 节省 Token | 说明 |
|--------|------|------|-----------|------|
| **P0** | 剥离 `reasoning_content` (BUG) | 🟢 低 | 1-2K/轮 | **不是优化，是修 bug** |
| **P1** | 压缩 HOT_TOOLS 为摘要 | 🟡 中 | 0-2,500 | 完整 teach 文档不必要 |
| **P1** | 压缩 i_rs 参数描述 | 🟢 低 | ~200 | 消除 4 次重复 |
| **P2** | 精简 TOOL_INDEX | 🟢 低 | ~400 | 仅保留工具名列表 |
| **P2** | {{SKILLS}} 摘要模式 | 🟡 中 | 变化 | 仅影响有安装技能的用户 |
| **P3** | 降低 recent_keep / 压缩阈值 | 🟢 低 | 渐进式 | 长对话场景 |

**P0 修复是关键 bug 修复（deepseek-reasoner 多轮对话失败），非可选优化。**
