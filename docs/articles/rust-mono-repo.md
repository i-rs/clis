# 70 工具 · 一柄利刃 — Rust 单体仓库工程实践

## 不止是 70 个 CLI

从第一次运行 `cargo build` 到拥有 70 个 CLI 工具，i-rs 走过的路是一个关于工程化思考的故事。

70 个工具意味着什么？

- 70 个 `Cargo.toml`
- 70 个 `main.rs`
- 70 组 `add/delete/list/get/update` CRUD 命令
- 70 套存储模型
- 70 份 SKILL.md 技能文件

如果各自为政，这将是一场维护噩梦。但在 i-rs 中，**创建一个新工具只需不到一分钟**，且保证零错误零警告。

秘密在于——**宏工程化**。

## 三驾马车：i-rs-core 的宏系统

`i-rs-core` 是 70 个工具的共享基石，它提供三个核心宏，构成了整个项目的骨架：

### `create_store!` — 一行代码搞定持久化

```rust
i_rs_core::create_store!(WeightStore, "weight");
```

这一行宏展开为：

- `WeightStore` 结构体，包装 `BTreeMap<String, WeightEntry>`
- `load_store()` 函数，从 `~/.config/i-rs/weight.json` 读取数据
- `save_store()` 函数，将数据写回磁盘
- 序列化/反序列化逻辑
- 错误处理

**70 个工具 = 70 行 storage 代码。** 这不是简写，而是一种架构决策——每个工具的存储层都经过相同的验证，拥有相同的可靠性。

### `skill_command!` — AI 技能系统的基石

```rust
i_rs_core::skill_command!("i-rs-weight");
```

这一行宏嵌入整个 SKILL.md 文件，并生成 7 个子命令：`info`、`search`、`teach`、`install`、`summary`、`content`、`raw`。它为每个工具赋予了 AI 自我解释能力。

### `exit_on_error!` — 统一错误处理

```rust
i_rs_core::exit_on_error!(run(cli.command, format), cli.json);
```

这一行宏确保：

- 所有错误都经过统一格式化
- JSON 模式下输出结构化错误
- 终端模式下输出可读友好的错误信息
- 进程退出码正确

## 模板化的 crate 结构

每个 i-rs 工具都遵循同一套严格的目录结构：

```
crates/i-rs-{name}/
├── src/
│   ├── main.rs           # Cli::parse() + exit_on_error!
│   ├── commands/         # add, delete, list, get, update, etc.
│   │   ├── mod.rs        # pub use 导出
│   │   ├── add.rs        # 新增条目
│   │   ├── delete.rs     # 删除条目
│   │   ├── list.rs       # 列出条目
│   │   ├── get.rs        # 查看条目
│   │   ├── update.rs     # 更新条目
│   │   ├── data.rs       # export/import/clear
│   │   └── skill.rs      # skill_command! 宏
│   ├── models/
│   │   └── mod.rs        # Entity + Store (BTreeMap)
│   ├── storage/
│   │   └── mod.rs        # create_store! 宏
│   └── presentation/
│       └── mod.rs        # render_table + 自定义格式化
├── Cargo.toml            # 极简依赖
└── README.md
```

这种一致性带来的好处远超预期：

- **知识迁移成本为零** — 熟悉一个工具就等于熟悉了所有工具
- **批量重构成为现实** — 一次 `search_replace` 遍历 70 个文件
- **代码审查标准化** — 每个文件该有什么、不该有什么，一目了然
- **自动化成为可能** — 新工具创建可以写成脚本

## BTreeMap 的偏执

i-rs 的所有存储都使用 `BTreeMap<String, Entity>`，而非 `HashMap`。这个看似微小的决策，影响深远：

- **确定性遍历顺序** — 每次输出顺序一致，对用户和 AI 都更可预测
- **可排序** — 天然支持按 key 排序显示
- **可差分** — `git diff` 时可以看到稳定的顺序变化

这不是性能妥协，而是一种有序设计的执念。

## 从 CI 到发布的全链路自动化

### 每一次推送

`.github/workflows/check.yml` 确保：

```
cargo check        → 0 errors, 0 warnings
cargo clippy       → -D warnings 严格模式
cargo fmt --check  → 代码格式统一
cargo deny check   → 依赖安全审计
```

### 每一次发布

通过 `cargo-dist` 自动化：

- **原生二进制** — macOS (x86 + ARM)、Linux、Windows 全平台分发
- **Homebrew Tap** — `brew install i-rs/tap/i-rs-kv`
- **npm 发布** — 通过 npm 分发原生二进制
- **GitHub Release** — 自动生成 changelog 和 release notes

一次 `git tag`，一切自动完成。

## 关于规模的思考

70 个工具体量的项目，如果每个工具独立仓库，光是 CI 配置就要写 70 份。单体仓库的选择让 i-rs 获得了：

| 维度 | 独立仓库 | 单体仓库 |
|------|---------|---------|
| CI 配置 | 70 份 | 1 份 |
| 依赖版本 | 各自管理 | 集中管理 |
| 核心库升级 | 逐个更新 | 一次编译 |
| 跨工具重构 | 几乎不可能 | 脚本批量处理 |
| 新工具创建 | 半小时 | 一分钟 |

这不是关于"哪个更好"的争论，而是关于在特定规模下"哪个更合理"的务实选择。

## 小结

i-rs 的 Rust 工程实践可以总结为三个关键词：

1. **宏驱动** — 用宏消除样板代码，让开发聚焦业务逻辑
2. **模板先行** — 严格的目录结构和代码规范，让一致性成为默认
3. **自动化闭环** — 从代码提交到二进制分发，全链路自动化

这不是一个关于"7000 行宏.rs"的故事，而是一个关于**如何用工程化思维管理 70 个 CLI 工具**的实践记录。
