# 70 个 CLI 工具，全都能教 AI 自己怎么用

> 一个 Rust 宏，70 个工具，每个都内置了 AI 技能系统。

## 先看这个

```bash
# 你是一个 AI，第一次见到这个 CLI
$ i-rs-kv --help
...
skill    AI skill system: run 'skill teach' for a complete
         AI guide, or 'skill info' for tool metadata
```

看到 `skill` 了吗？接下来你只需要：

```bash
# AI 自己教自己
$ i-rs-kv skill teach
```

立刻拿到一份专为 AI 设计的教学文档——包含工具身份、数据存储位置、REST API 路径、全部命令参考、生态上下文。

全程零人工参与。一个新的 CLI，AI 在 10 秒内从陌生变成精通。

---

这不是概念设计。这是 **i-rs**——一个开源 Rust CLI 工具集，70 个工具，已经全部实现。

## 为什么这很重要？

AI 时代，CLI 工具需要学会和 AI 对话。

传统 CLI 的 `--help` 是给人看的。AI 读到 `--help`，看到的是一堆参数和缩写，还得自己去猜数据存哪了、有哪些命令组合、错误怎么处理。

i-rs 改变了这个范式：

```
AI 读到 --help → 发现 skill 命令 → 运行 skill teach → 获得完整教学文档 → 熟练使用工具
```

每个步骤都是工具设计的**一部分**，而不是事后加的文档。

## 技术上说，怎么做到的？

一个声明式宏：

```rust
// crates/i-rs-xxx/src/commands/skill.rs
i_rs_core::skill_command!("i-rs-kv");
```

这一行宏展开为：

- 7 个子命令：`info`、`search`、`teach`、`install`、`summary`、`content`、`raw`
- 完整的命令处理逻辑
- 编译时通过 `include_str!` 嵌入 SKILL.md 技能文件

意味着零运行时开销、永远同步、离线可用。

## 不止是 AI 友好

i-rs 的每个工具都是完整的 CRUD 应用：

```bash
# 记录体重
i-rs-weight add --weight 72.5

# 查看本月支出
i-rs-ledger list --month 2026-05

# 检查宠物喂食记录
i-rs-feedpet list

# 统计投资回报率
i-rs-invest stats
```

同样的 `add` / `list` / `get` / `update` / `delete` 模式贯穿所有 70 个工具。学一个等于会全部。

## 还有 REST API

通过 `i-rs-api`，每个工具的能力都暴露为 HTTP 接口：

```bash
# 启动 API 服务
i-rs-api serve

# 用 curl 操作数据
curl -X POST http://localhost:8080/api/weight \
  -H "Content-Type: application/json" \
  -d '{"weight": 72.5}'
```

CLI 和 API 读写**同一个 JSON 文件**。不需要同步，不需要迁移，数据一致性天然保证。

## 还有一个浏览器扩展

对于 kv 这样的工具，还有 Chrome 扩展，通过 Native Messaging 直接调用本地二进制。在浏览器弹窗里查数据、改数据，不用开终端。

## 关于这个项目

- **70 个 CLI 工具**，覆盖健康、财务、家庭、学习、时间管理等 10+ 领域
- **Rust 单体仓库**，`cargo check` = 0 errors, 0 warnings
- **全平台**：macOS / Linux / Windows，通过 Homebrew 和 npm 分发
- **AGPL-3.0 开源**，欢迎贡献

```bash
# 安装
brew install i-rs/tap/i-rs-kv

# 或者从 GitHub Releases 下载
```

---

**[GitHub: i-rs/clis](https://github.com/i-rs/clis)**

AI 不会等你写文档。让你的 CLI 学会自我介绍。
