# i-rs 内置 Skill 系统 — CLI 工具的 AI 自我介绍

## 一个让 AI 自学成才的命令

每个 i-rs CLI 工具的 `--help` 输出中，都有这样一行：

```
skill    AI skill system: run 'skill teach' for a complete
         AI guide, or 'skill info' for tool metadata
```

这意味着：**每一个 i-rs 工具都内置了一套完整的 AI 技能系统。** 无需外部配置文件，无需手动安装插件，无需翻阅文档——AI Agent 读到 `--help` 的那一刻，就已经被告知如何学习这个工具。

这是 i-rs 最独特的设计：CLI 工具不再只是给人用的，它们从一开始就被设计成**同样能被 AI 理解**。

## 技能文件的编译时嵌入

i-rs 的 skill 文件采用 **AgentSkills 标准格式**——YAML 前置元数据 + Markdown 正文，这是 Claude Code、OpenAI Codex、Cursor、VS Code Copilot、Gemini CLI 等主流 AI Agent 共同支持的通用技能格式。

每个工具的 skill 文件位于：

```
skills/i-rs-{name}/SKILL.md
```

在编译时，`skill_command!` 宏通过 `include_str!` 将这个文件直接嵌入到二进制中：

```rust
// 宏内部展开的代码
const SKILL_RAW: &str =
    include_str!(concat!("../../../../skills/", $crate_name, "/SKILL.md"));
```

这意味着：

- **零运行时开销** — 技能文件是二进制的一部分，无需磁盘 IO
- **永远同步** — 技能内容与代码版本绑定，不会过期
- **跨平台一致** — macOS、Linux、Windows 体验完全一样
- **离线可用** — 不依赖网络，不依赖外部存储

## 完整的 skill 命令家族

`skill_command!` 宏为每个工具生成了一个完整的子命令树，共 7 个子命令：

```
i-rs-xxx skill
├── info        显示结构化元数据（名称、描述、命令列表）
├── search      在技能内容中搜索关键词
├── teach       生成 AI 教学文档
├── install     安装技能文件到指定目录
├── summary     显示工具概要描述
├── content     显示 YAML 后的正文内容
└── raw         显示原始 SKILL.md
```

### skill info — 工具元数据查询

对于 AI 来说，这是快速了解一个工具最快的方式：

```bash
$ i-rs-kv skill info
Tool: i-rs-kv
Description: Key-Value storage CLI - store and retrieve simple key-value data

Commands (12 total):
  add - Add a new key-value entry
  delete - Delete a key-value entry
  list - List all entries (with optional pattern matching)
  get - Get a value by key
  update - Update a key-value entry
  search - Search entries by value or key pattern
  stats - Show storage statistics
  copy - Copy an entry to a new key
  rename - Rename an entry key
  example - Show usage examples
  data - Data management commands
  skill - AI skill system commands
```

所有子命令都被结构化地列出来，AI 可以据此构建工具调用计划。

### skill teach — AI 专属教学文档

这是整个 skill 系统的核心功能。它会生成一份**专门为 AI 设计的教学文档**，结构如下：

```
# {工具名} — AI Teaching Document

## Tool Identity
{工具的完整描述}

## Data Storage
- Config file: ~/.config/i-rs/{数据文件名}.json
- Format: JSON with BTreeMap structure
- Override with CONFIG_DIR environment variable

## REST API Integration
- REST API available via i-rs-api server
- Default endpoint: http://localhost:8080
- Path: /api/{工具名}

## Command Reference
{完整的命令列表}

## Ecosystem
- Part of the i-rs CLI toolset (~70 tools)
- Shared library: i-rs-core (Storage<T>, macros, presentation)
- AI skill format: AgentSkills
- All commands support --json for structured output
- Error handling uses anyhow::Result pattern
```

AI 将这份文档作为系统提示的一部分后，就能从以下维度全面理解该工具：

1. **身份** — 它是什么，用来做什么
2. **存储** — 数据在哪里，什么格式，如何访问
3. **API 集成** — 如果通过 REST API 访问，路径是什么
4. **命令参考** — 每个命令的作用
5. **生态上下文** — 在 i-rs 全家桶中的位置

### skill search — 内容级检索

当 AI 需要查找技能文件中的特定细节时：

```bash
$ i-rs-read skill search "daily reading"
45:    daily reading progress or reading streak.
92:  - `daily_reading`: Track daily reading progress.
```

行号输出让 AI 可以精确引用内容。

### skill install — 技能文件分发

在需要将技能文件安装到 AI Agent 的配置目录时：

```bash
# 输出到终端（默认）
i-rs-kv skill install

# 安装到指定目录
i-rs-kv skill install ~/.claude/skills/

# 指定 AI Agent（预设安装路径）
i-rs-kv skill install --agent openclaw
```

## 一个宏，注入 70+ 工具

`skill_command!` 是一个声明式宏，所有 70+ i-rs 工具都通过**一行代码**接入 skill 系统：

```rust
// crates/i-rs-{name}/src/commands/skill.rs
i_rs_core::skill_command!("i-rs-xxx");
```

宏展开后生成：

- `SkillCommand` 枚举（7 个变体，每个带 clap 属性）
- `handle_skill()` 函数（完整的命令处理逻辑）
- 编译时嵌入的 `SKILL_RAW` 常量

这个宏是整个系统的基石。它保证了：

| 维度 | 效果 |
|------|------|
| 一致性 | 所有工具的 skill 接口完全统一 |
| 复用性 | 核心逻辑在一处维护 |
| 可靠 性 | 宏展开的代码经过全面测试 |
| 简洁性 | 每个工具只需 1 行代码 |

## AgentSkills 标准兼容

i-rs 的 SKILL.md 遵循 AgentSkills 标准。一个标准的技能文件如下：

```yaml
---
name: i-rs-kv
description: Key-Value storage CLI - store and retrieve simple key-value data
---

# i-rs-kv

Manage key-value pairs for configuration and data storage.

## Commands

### add
Add a new key-value entry.

Usage: `i-rs-kv add <key> <value> [--tags <tags>]`

### get
Get a value by key.

Usage: `i-rs-kv get <key>`
...
```

这种格式被以下平台原生支持：

- **Claude Code** — 自动识别 `.claude/skills/` 下的 SKILL.md
- **OpenAI Codex** — 支持 AgentSkills 格式的自动加载
- **VS Code Copilot** — 通过 skills 目录注入上下文
- **Cursor** — 兼容 AgentSkills 规范
- **Gemini CLI** — 支持标准 SKILL.md 格式
- **Qoder** — 支持 SKILL.md 加载

这意味着：**一个技能文件通行所有主流 AI Agent。**

## AI 与 i-rs 工具的交互流程

当一个 AI Agent 遇到 i-rs 工具时，推荐的工作流是：

```
步骤 1: AI 读取 --help
        → 发现 skill 命令
        → 知道这是一个有 AI 技能系统的工具

步骤 2: AI 运行 skill info
        → 获取结构化元数据
        → 了解所有子命令

步骤 3: AI 运行 skill teach
        → 获取完整教学文档
        → 理解存储、API、生态

步骤 4: AI 开始使用工具
        → 基于教学文档执行具体操作
```

整个过程无需人类参与，AI 可以在几秒钟内从一个陌生工具变成熟练用户。

## design philosophy: teach, not configure

i-rs 的 skill 系统设计哲学可以概括为一句话：

&gt; **教 AI 使用工具，而不是让 AI 配置工具。**

大多数 CLI 工具靠 AI 自己去理解 `--help` 输出、README 文档、甚至是源码来学习如何使用。这不仅效率低下，而且容易出错。

i-rs 的 skill 系统改变了这一点：

- 工具主动告诉 AI "我有 skill 系统"
- AI 运行 `skill teach` 获得结构化的教学材料
- AI 获得深度语义理解——不仅知道有哪些命令，还知道数据存在哪里、API 如何调用

这是一种**主动教学**而非**被动配置**的设计思路。

## 小结

i-rs 的 skill 系统不是给"技能文件"加了个 CLI 入口，而是从根本上重新思考了 CLI 工具与 AI 之间的关系。

核心创新点：

1. **编译时嵌入** — 技能文件随二进制分发，零运行时开销
2. **宏驱动** — 一行宏为 70+ 工具注入完整的 skill 子系统
3. **AgentSkills 标准** — 兼容所有主流 AI Agent
4. **AI 教学文档** — `skill teach` 专为 AI 设计，包含存储、API、生态上下文
5. **从 --help 开始** — 即使没有安装任何技能文件，AI 读取 `--help` 就知道该怎么做

在 AI 时代，CLI 工具需要的不仅是人类可读的帮助信息，更需要 AI 可理解的语义元数据。i-rs 的 skill 系统，就是对此的完整回答。
