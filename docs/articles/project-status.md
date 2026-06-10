# i-rs CLI 项目状态报告

&gt; 70 个 CLI 工具、114 次提交、0 个编译警告——一个人的 Rust 项目走到了哪里？

## 概述

**i-rs** 是一个 Rust 编写的跨平台 CLI 工具集，目前包含 **70 个独立工具**，覆盖个人数据管理的方方面面——从体重记录到域名到期提醒，从宠物喂食到投资管理。

项目采用 Cargo 单体仓库（workspace）管理，总计 **72 个 crate**（含 i-rs-core 核心库和 i-rs-api REST API 服务），代码量约 **116,000 行**，全部由一人完成。

```bash
cargo check    → 0 errors
cargo clippy   → 0 warnings（-D warnings 严格模式）
cargo test     → 21 passed（核心库单元测试）
```

## 项目结构

```
crates/         → 72 个 crate
  i-rs-core/      共享核心库（Storage、宏、展示、校验）
  i-rs-api/       REST API 服务（Axum）
  i-rs-{name}...   70 个 CLI 工具
skills/         → 71 份 SKILL.md（编译时嵌入每个二进制）
docs/           → VitePress 文档站
  articles/      推广与技术文章（11 篇）
extensions/     → Chrome 扩展 + Native Messaging 宿主
.github/        → CI/CD 全自动化
```

70 个工具按领域分布：

```
健康管理  │ weight / height / sleep / mood / exercise / run / cycling
          │ water / step / dose / fast / allergy / cal / habit / cycle / sit / vision
财务管理  │ ledger / budget / invest / debt / recur / invoice / tax / goal / sub / kv / keys
家庭维护  │ appliance / sheet / toothbrush / towel / bed / filter / purify / ac / plant / aqua
宠物照护  │ feedpet / petbath / walkdog
学习记录  │ read / vocab / article / spark / snippet / quote
社交关系  │ contact / birthday / gift
时间管理  │ remind / event / time / tick / deploy
其他工具  │ domain / bestby / todo / project / grocery / meal / pig / car
          │ password / note / bookmark / server / ... 共 70 个
```

## 近期关键进展

### AI Skill 系统（核心特色）

每个 CLI 工具都内置了完整的 AI 技能系统，通过 `skill_command!` 宏注入：

```
──── i-rs-kv --help ──────────────────────────────
...
skill    AI skill system: run 'skill teach' for a
         complete AI guide, or 'skill info' for tool metadata
──── ─────────────────────────────────────────────
```

7 个子命令：`info`、`search`、`teach`、`install`、`summary`、`content`、`raw`

AI Agent 的工作流：
```
读取 --help → 发现 skill 命令 → 运行 skill teach → 获得教学文档 → 熟练使用
```

技术实现：一个声明式宏 + `include_str!` 编译时嵌入。70 个工具 = 70 行 skill 代码。

### 文章体系搭建

`docs/articles/` 推广文章专区已上线，共 **11 篇文章**：

- **3 篇推广文** — 可直接发布于掘金、dev.to、V2EX、Medium
- **5 篇技术文** — AI Skill 系统、共享存储层、Rust 工程实践、全栈覆盖、数据指挥中心
- **1 篇内置 Skill 系统专文** — 深入宏实现与 AgentSkills 标准
- **1 篇项目状态报告**（本文）

`docs/spec/index.md` 新增 **§16 文章与推广规范**，定义何时写文章、写什么、发哪里。

### 架构优化

此前完成了系统性的代码质量提升：

- `presentation!()` 宏统一 26 个 crate 的展示层
- `data_command!` 宏统一 data 子命令
- 移除 27 个文件的 `#![allow(dead_code)]`，移除 9 个 crate 的 `#![allow(clippy::all)]`
- `i-rs-api` 取消实验性标签，转为正式模块
- REST API 补齐缺失端点

## 技术栈

| 层 | 技术 |
|----|------|
| 语言 | Rust（2024 edition） |
| CLI 框架 | clap（derive macro） |
| 存储 | 纯 JSON 文件（`~/.config/i-rs/*.json`） |
| API 服务 | Axum + Tokio + tower-http |
| 序列化 | serde + serde_json |
| 表格输出 | tabled |
| 颜色输出 | owo-colors |
| 日期处理 | chrono |
| 自动化 | cargo-dist（CI/CD + 发布） |
| 文档 | VitePress |

## 分发渠道

- **全平台二进制** — macOS (x86 + ARM) / Linux / Windows
- **Homebrew Tap** — `brew install i-rs/tap/i-rs-{name}`
- **npm** — npm 分发原生二进制
- **GitHub Releases** — cargo-dist 全自动发布，打 tag 即发布

## 一点数字

| 指标 | 数值 |
|------|------|
| CLI 工具数 | 70 |
| 总 crate 数 | 72 |
| 总提交数 | 114 |
| 总代码行数 | ~116,000 |
| 作者 | 1 人 |
| 编译检查 | 0 errors, 0 warnings |
| 核心库单元测试 | 21 passed |
| 技能文件 | 71 份 |
| 推广文章 | 11 篇 |
| 许可证 | AGPL-3.0 |

## 下一步

项目目前处于"工具覆盖已基本完整，但需要持续打磨"的阶段。主要方向：

- 更多工具的独特命令支持（chart、stats、calendar 等）
- i-rs-api 端点完善
- 社区贡献文档和模板
- 第三方集成（Alfred、Raycast、macOS Shortcuts）

---

**[GitHub: i-rs/clis](https://github.com/i-rs/clis)**

一个人的项目，等着变成很多人的项目。
