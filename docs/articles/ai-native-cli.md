# AI 原生命令行 — i-rs 的 AI Skill 系统

## 当 CLI 学会自我介绍

想象这样一个场景：一个 AI Agent 第一次见到你的工具，它做的第一件事是什么？

对于大多数工具来说，答案是——不知道从何下手。AI 需要阅读文档、分析源码、理解参数，这中间充满了猜测和试错。

但对于 i-rs CLI 工具集，答案只有一行命令：

```bash
i-rs-kv --help
```

输出中赫然写着：

```
skill    AI skill system: run 'skill teach' for a complete
         AI guide, or 'skill info' for tool metadata
```

就在这一刻，AI 就知道了三条关键信息：

1. **这个 CLI 内置了 AI 技能系统**
2. **运行 `skill teach` 可以获得完整的教学文档**
3. **运行 `skill info` 可以查看工具元数据**

这不是巧合，这是设计使然。

## 一个宏，七十把剑

i-rs 的 AI Skill 系统并非某个工具的专属功能，而是通过一个 Rust 宏 `skill_command!` 注入到所有 70+ 工具中。

每个工具在编译时就把自己的 SKILL.md 技能文件嵌入到二进制中。这意味着：

- **零运行时依赖** — 技能文件随二进制分发，无需额外下载
- **永远同步** — 技能内容随代码版本更新，不会过时
- **跨平台一致** — Windows、macOS、Linux 体验完全一致

## 技能命令的全貌

### `skill info` — 快速了解工具

以 `i-rs-kv` 为例，运行 `skill info` 会输出：

```
Tool: i-rs-kv
Description: Key-Value storage CLI - store and retrieve simple key-value data

Commands (12 total):
  add - Add a new key-value entry
  get - Get a value by key
  list - List all entries (with optional pattern matching)
  search - Search entries by value or key pattern
  stats - Show storage statistics
  copy - Copy an entry to a new key
  rename - Rename an entry key
  ...
```

这对于 AI 来说是完美的"工具说明书"——结构化、无冗余、一步到位。

### `skill teach` — AI 教学文档

这是 AI Skill 系统的灵魂功能。运行后会生成一份**专为 AI 设计的教学文档**，包含：

- **工具身份** — 这是什么工具，用来做什么
- **数据存储** — 数据存在哪里，什么格式，如何访问
- **REST API 集成** — 如果通过 i-rs-api 访问该工具，接口路径是什么
- **命令参考** — 完整的命令列表和说明
- **生态上下文** — 这个工具在 i-rs 生态中的位置

AI 只需要将这份文档作为系统提示的一部分，就能立即掌握整个工具的使用方法。

### `skill search` — 内容级检索

当 AI 对工具已有基本了解，但需要查找某个特定细节时：

```bash
i-rs-read skill search "daily reading"
```

返回匹配内容及行号，让 AI 精准定位。

### `skill install` — 技能文件分发

将 SKILL.md 安装到指定目录，支持 `--agent` 参数预设主流 AI Agent 的安装路径：

```bash
i-rs-kv skill install --agent openclaw
```

这使得技能文件可以轻松集成到 Claude Code、OpenAI Codex、Cursor 等任意支持 AgentSkills 标准的 AI 工具中。

## AgentSkills 标准—通用语言

i-rs 的 SKILL.md 遵循 **AgentSkills** 标准——一种被 Claude Code、OpenAI Codex、Cursor、VS Code Copilot、Gemini CLI 等主流 AI Agent 普遍认可的格式。

这意味着：

- **无需格式转换** — 一个 SKILL.md 适用于所有 AI Agent
- **标准即生态** — 你的技能不仅对你自己有用，对整个 AI 生态都有价值
- **未来兼容** — 只要 AI Agent 支持 AgentSkills，i-rs 的技能就能工作

## 从 --help 到技能掌握，只需一个命令

这是 i-rs 最引以为傲的设计哲学：

> **每一个 CLI 工具，首先应该告诉 AI 如何理解自己。**

传统的 CLI 设计只考虑人类用户——`--help` 展示参数，README 讲解用法。但在 AI 时代，这是不够的。AI 需要结构化的、语义丰富的元数据，需要知道数据存在哪里、API 如何调用、命令之间如何组合。

i-rs 的 AI Skill 系统填补了这个空白。

它的工作流优雅而简洁：

```
AI 读取 --help → 发现 "skill" 命令 → 运行 skill teach → 获得完整教学文档 → 精通工具
```

全程无需人类干预，无需翻阅外部文档，无需猜测。

## 这为什么重要？

因为**AI 时代的 CLI 工具，需要学会和 AI 对话**。

传统 CLI 的"人机交互"正在演进为"人 — AI — 工具"的三元交互。在这个新范式下，工具不仅要被人类理解，更要被 AI 理解。

i-rs 用一套优雅的宏系统，为 70 个工具赋予了"自我介绍"的能力。这不仅是技术上的创新，更是 CLI 设计理念的前瞻探索。

当你运行 `i-rs-weight skill teach`，AI 不仅学会了如何记录体重，还理解了数据存储位置、图表生成方式、统计计算方法——它获得的是对一个工具的**深度语义理解**。
