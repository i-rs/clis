# TUI 消息滚动异常 — 排查分析与修复方案

> 日期: 2026-05-20  
> 范围: i-rs-claw TUI (`ui/chat.rs`, `tui/main_loop.rs`, `app.rs`)  
> 症状: 滚动时部分消息"消失"，表现不稳定

---

## 1. 排查结论

发现 **2 个 Bug**，其中 Bug 1 是导致"消息消失"的直接原因。

---

## 2. Bug 1 (Critical): 消息高度估算与实际渲染不一致

### 2.1 根因

`render_chat` 使用三遍渲染算法：
- **Pass 1**: 用 `message_line_count()` 估算每条消息的行数
- **Pass 2**: 从最新消息开始累加，确定能显示多少条
- **Pass 3**: 用 `build_message_item()` 实际渲染消息

**问题**: Pass 1 的估算 和 Pass 3 的实际渲染使用了**不同的逻辑**。

### 2.2 代码对比

**Pass 1 — `message_line_count()` (chat.rs:356-358)**:
```rust
Message::Assistant { text } => {
    // 简单文本换行估算，宽度 = text_width
    1 + wrapped_line_count(text, text_width) + 1
}
```

`wrapped_line_count` 做的事：去 ANSI → 按 `\n` 分割 → 每行按 `text_width` 计算换行数。

**Pass 3 — `build_message_item()` (chat.rs:602-632)**:
```rust
Message::Assistant { text } => {
    // Markdown 渲染，宽度 = text_width - 3
    let md_lines = render_markdown(text, text_width.saturating_sub(3));
    // ...
}
```

`render_markdown` 做的事：
1. 解析 Markdown 语法（标题、列表、代码块、粗体等）
2. 使用**不同的宽度** (`text_width - 3`)
3. 产生**额外行**：代码块分隔线、列表符号 `•`、标题 `#` 等

### 2.3 具体差异

| 因素 | `message_line_count` 估算 | `build_message_item` 实际 | 差异 |
|------|--------------------------|--------------------------|------|
| 宽度 | `text_width` | `text_width - 3` | 实际更窄 → 更多换行 |
| Markdown | 不解析，纯文本换行 | 完整解析 | 代码块/列表/标题增加行数 |
| 代码块 | 按普通文本计算 | 添加 `── code ──` 分隔线 + 背景 | 多 2 行 |
| 列表 | 按普通文本计算 | 每行前加 `• ` | 可能多换行 |

### 2.4 复现场景

```
假设 viewport 可显示 20 行，有 5 条消息：

消息 A (User):     2 行  → 估算 2，实际 2  ✅
消息 B (Assistant): 8 行  → 估算 6（纯文本），实际 10（Markdown） ❌
消息 C (User):     2 行  → 估算 2，实际 2  ✅
消息 D (Assistant): 4 行  → 估算 3（纯文本），实际 6（Markdown） ❌
消息 E (User):     2 行  → 估算 2，实际 2  ✅

Pass 1 估算: A(2) + B(6) + C(2) + D(3) + E(2) = 15 ≤ 20 → 全部显示
Pass 3 实际: A(2) + B(10) + C(2) + D(6) + E(2) = 22 > 20 → 消息 A 被挤出！
```

**结果**: 用户看到消息 A "消失"了，但没有任何滚动操作。

### 2.5 修复方案

**方案 A (推荐): 统一使用实际渲染计算行数**

在 Pass 1 中，对 Assistant 消息也调用 `render_markdown` 来计算实际行数：

```rust
// message_line_count() 中 Assistant 分支改为:
Message::Assistant { text } => {
    let header_lines = 1; // "Claw:" 标题
    let trailing = 1;     // 空行分隔
    let body_lines = if text.is_empty() {
        1 // "..."
    } else {
        let md_lines = render_markdown(text, text_width.saturating_sub(3));
        md_lines.len()
    };
    header_lines + body_lines + trailing
}
```

**方案 B: 在 Pass 1 中预渲染并缓存**

将 Pass 1 和 Pass 3 合并——先渲染所有消息到缓存，再从缓存中计算哪些能显示：

```rust
// 预渲染所有可见消息
let mut rendered: Vec<(usize, ListItem<'static>, usize)> = Vec::new();
for (rev_idx, msg) in app.messages.iter().rev().enumerate() {
    if rev_idx < app.scroll_offset { continue; }
    let msg_index = app.messages.len() - 1 - rev_idx;
    let item = build_message_item(app, msg, text_width, msg_index, &mut format_cache);
    let height = item.height(); // ratatui ListItem 有 height() 方法
    rendered.push((rev_idx, item, height));
}

// 从新到旧累加，确定哪些能显示
// ...
```

**方案 A 改动最小，推荐优先实施。**

---

## 3. Bug 2 (Medium): 滚动锚定策略导致消息跳变

### 3.1 根因

当前滚动机制以**最新消息**为锚点，从新到旧填充 viewport。当 `scroll_offset` 变化时，锚点消息改变，导致整个可见消息集合重新计算。

### 3.2 当前逻辑

```rust
// app.rs
pub fn scroll_up(&mut self) {
    self.scroll_offset += 1;  // 跳过更多新消息
}
pub fn scroll_down(&mut self) {
    self.scroll_offset = self.scroll_offset.saturating_sub(1);
}
```

```rust
// ui/chat.rs render_chat
for (rev_idx, msg) in app.messages.iter().rev().enumerate() {
    if rev_idx < app.scroll_offset { continue; }  // 跳过 scroll_offset 条新消息
    // ...
}
// 然后从剩余消息中，从新到旧填充 viewport
```

### 3.3 问题表现

```
初始状态 (scroll_offset=0，viewport 可显示 3 条):
┌────────────┐
│ 消息 E (新) │  ← 锚点
│ 消息 D     │
│ 消息 C     │
└────────────┘
(消息 A、B 在上方被裁剪)

按一次 Up (scroll_offset=1):
┌────────────┐
│ 消息 D (新) │  ← 新锚点
│ 消息 C     │
│ 消息 B     │  ← 消息 E "消失"了
└────────────┘
(消息 A 仍在上方被裁剪)
```

如果消息 D 比消息 E 高很多，可能连消息 C 都被挤出：

```
按一次 Up (scroll_offset=1)，但消息 D 很高:
┌────────────┐
│ 消息 D (新) │  ← 新锚点，占满 viewport
│ (D 的第二部分)│
│ (D 的第三部分)│  ← 消息 C、E 都"消失"了
└────────────┘
```

### 3.4 修复方案

**方案 A: 行级滚动（推荐）**

将 `scroll_offset` 从"跳过的消息数"改为"跳过的行数"：

```rust
// app.rs
pub struct App {
    // 改为: 从底部跳过的行数 (0 = 底部)
    pub scroll_offset_lines: usize,
}
```

渲染时：
1. 先计算所有消息的实际行数（从新到旧）
2. 跳过底部 `scroll_offset_lines` 行
3. 从剩余部分填充 viewport

```rust
// ui/chat.rs render_chat
let mut all_lines: Vec<ListItem<'static>> = Vec::new();
let mut heights: Vec<usize> = Vec::new();

for (rev_idx, msg) in app.messages.iter().rev().enumerate() {
    let msg_index = app.messages.len() - 1 - rev_idx;
    let item = build_message_item(app, msg, text_width, msg_index, &format_cache);
    let h = item.height();
    all_lines.push(item);
    heights.push(h);
}

// 从底部跳过 scroll_offset_lines 行
let mut skipped = 0;
let mut start_idx = 0;
for (i, &h) in heights.iter().enumerate() {
    if skipped + h <= app.scroll_offset_lines {
        skipped += h;
        start_idx = i + 1;
    } else {
        break;
    }
}

// 从 start_idx 开始填充 viewport
// ...
```

**方案 B: 保持消息级滚动，但锚定到当前可见的最旧消息**

滚动时不改变锚点，而是将当前最旧可见消息保持在同一屏幕位置：

```
初始:
┌────────────┐
│ 消息 E     │
│ 消息 D     │
│ 消息 C     │  ← 最旧可见
└────────────┘

按 Up 后（锚定消息 C 在相同位置）:
┌────────────┐
│ 消息 D     │
│ 消息 C     │  ← 仍在同一位置
│ 消息 B     │  ← 新出现的
└────────────┘
```

**方案 A 更符合用户对"滚动"的直觉预期，推荐。**

---

## 4. Bug 3 (Low): 流式更新时消息跳动

### 4.1 根因

当 Assistant 消息正在流式输出时，消息内容不断增长，行数增加。由于 viewport 始终从最新消息开始填充，增长的消息会把旧消息挤出视口。

### 4.2 表现

```
流式输出前:
┌────────────┐
│ 消息 E (空) │  ← 即将开始流式
│ 消息 D     │
│ 消息 C     │
└────────────┘

流式输出中 (消息 E 增长到占满 viewport):
┌────────────┐
│ 消息 E     │
│ (更多内容)  │
│ (更多内容)  │  ← 消息 C、D 被挤出
└────────────┘
```

这是当前设计的**预期行为**（新消息优先可见），但对用户来说可能感觉"旧消息消失了"。

### 4.3 修复方案

**方案 A: 流式输出时自动跟随滚动**

当用户在底部（`scroll_offset == 0`）时，自动保持最新消息可见。这已经是当前行为，无需修改。

**方案 B: 流式输出时锁定滚动位置**

如果用户已向上滚动查看历史，流式输出不应改变用户的滚动位置。需要记录"当前视口顶部对应的消息索引和行偏移"，而非简单的 `scroll_offset`。

---

## 5. 修复优先级

| 优先级 | Bug | 影响 | 改动量 |
|--------|-----|------|--------|
| **P0** | Bug 1: 高度估算不一致 | 消息无故消失 | 小（改 `message_line_count`） |
| **P1** | Bug 2: 滚动锚定策略 | 滚动时消息跳变 | 中（改滚动机制） |
| **P2** | Bug 3: 流式跳动 | 旧消息被挤出 | 小（可选优化） |

---

## 6. 实施计划

### Phase 1: 修复 Bug 1（立即，0.5 天）

修改 `ui/chat.rs` 中 `message_line_count` 的 Assistant 分支：

```rust
Message::Assistant { text } => {
    let header_lines = 1;
    let trailing = 1;
    let body_lines = if text.is_empty() {
        1
    } else {
        let md_lines = render_markdown(text, text_width.saturating_sub(3));
        md_lines.len()
    };
    header_lines + body_lines + trailing
}
```

同时修复 ToolCall 展开状态的行数估算（当前也未考虑 `format_json_result` 的实际渲染宽度差异）。

### Phase 2: 修复 Bug 2（1-2 天）

将 `scroll_offset` 从消息级改为行级：

1. `app.rs`: `scroll_offset: usize` → `scroll_offset_lines: usize`
2. `app.rs`: `scroll_up()` / `scroll_down()` 改为按行增减（如每次 3 行）
3. `ui/chat.rs`: `render_chat` 改为按行跳过
4. `tui/main_loop.rs`: 鼠标滚动改为按行而非按消息

### Phase 3: 优化 Bug 3（可选，0.5 天）

在流式输出期间，如果用户不在底部，保持其滚动位置不变。

---

## 7. 回归测试建议

修复后应验证以下场景：

| 场景 | 预期行为 |
|------|---------|
| 纯文本 Assistant 消息 | 高度估算 = 实际渲染 |
| Markdown Assistant 消息（含代码块/列表/标题） | 高度估算 = 实际渲染 |
| 按 Up 键滚动 | 每次向上移动固定行数，不跳变 |
| 按 Down 键滚动 | 每次向下移动固定行数，到底部自动归零 |
| 鼠标滚轮 | 方向正确，按行滚动 |
| 流式输出中滚动 | 用户位置不被新 token 强制改变 |
| 超长单条消息 | 可以逐行滚动查看完整内容 |
| 消息删除/新会话 | 滚动状态正确重置 |
