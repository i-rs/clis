# 70 个 CLI 工具，我是怎么用 Rust 管理过来的

&gt; 一个 Cargo 工作空间，70 个 crate，全部用宏生成。`cargo check` = 0 errors, 0 warnings。

## 问题

当你的 CLI 工具从一个变成 70 个，你面临的不再是"怎么写功能"，而是"怎么管理规模"。

70 个工具意味着：
- 70 个 `main.rs`
- 70 组 CRUD 命令
- 70 套 JSON 存储
- 70 份 AI 技能文件

如果各写各的，维护成本会爆炸。

## 解法：一个核心库 + 三个宏

### `create_store!`：一行代码搞定持久化

```rust
i_rs_core::create_store!(WeightStore, "weight");
```

这一行展开为 `load_store()`、`save_store()`、`export_data()`、`import_data()`、`clear_data()` 五个函数。操作 `~/.config/i-rs/weight.json`。

70 个工具 = 70 行 storage 代码。每个都经过了相同的验证，拥有相同的可靠性。

### `skill_command!`：一行代码注入 AI 技能

```rust
i_rs_core::skill_command!("i-rs-weight");
```

这一行展开为 7 个子命令（info / search / teach / install / summary / content / raw），编译时嵌入 SKILL.md。

70 个工具 = 70 行 skill 代码。每个都有一致的 AI 教学接口。

### `exit_on_error!`：一行代码统一错误处理

```rust
i_rs_core::exit_on_error!(run(cli.command, format), cli.json);
```

终端模式下输出可读错误，JSON 模式下输出结构化错误，退出码自动处理。

## 模板的力量

每个 crate 长一样：

```
crates/i-rs-xxx/
├── src/
│   ├── main.rs
│   ├── commands/  (add / delete / list / get / update / data / skill)
│   ├── models/    (Entity + BTreeMap Store)
│   ├── storage/   (create_store! 一行)
│   └── presentation/  (render_table)
├── Cargo.toml
└── README.md
```

好处：
- 熟悉一个 = 熟悉全部
- 批量重构一次搞定——改一个模式，sed 遍历 70 个
- 新 crate 创建 ≈ 一分钟
- 代码审查标准化——知道每个文件该有什么

## BTreeMap 的小偏执

所有存储用 `BTreeMap<String, Entity>`，不用 `HashMap`。

为什么？确定性遍历顺序。`list` 出来永远有序，`git diff` 看到的变化是可预期的。

小事，但在 70 个工具上积累的体验差异很大。

## CI/CD：一次提交，全链路自动化

```yaml
# 每次推送
cargo check       → 必须 0 errors 0 warnings
cargo clippy      → -D warnings 严格模式
cargo fmt --check → 格式统一
cargo deny check  → 依赖安全审计

# 每次打 tag
cargo-dist → 全平台二进制 + Homebrew + npm + GitHub Release
```

全部自动化。不需要手动打包、手动上传、手动写 changelog。

## 一点数字

| 指标 | 数值 |
|------|------|
| CLI 工具数 | 70 |
| 代码行数 | ~50,000 |
| `cargo check` | 0 errors, 0 warnings |
| `cargo clippy` | 0 warnings |
| 单元测试 | 21（i-rs-core）+ 各 crate |
| 存储格式 | 纯 JSON |
| 编译时间 | ~2 分钟（全量） |
| 单工具体积 | ~500KB |

## 想说的

宏不是炫技，是用代码生成对抗规模复杂度的务实选择。

模板不是偷懒，是让认知负荷不随工具数量增长的刻意设计。

自动化不是锦上添花，是让 70 个工具和一个工具一样容易维护的必要投入。

---

**[GitHub: i-rs/clis](https://github.com/i-rs/clis)**

Rust + 宏 + 模板 + 自动化 = 70 个工具，一份精力。
