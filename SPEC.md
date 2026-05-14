# i-rs CLI 项目规范

## 1. 项目结构

```
i-rs-clis/
├── crates/
│   ├── i-rs-core/              # 共享核心库
│   │   └── src/
│   │       ├── lib.rs          # 公共API导出
│   │       ├── macro.rs        # create_store!, skill_command!, exit_on_error!
│   │       ├── storage/        # 通用存储 (Storage<T>)
│   │       ├── presentation/   # 输出格式化
│   │       │   ├── mod.rs     # print函数 + OutputFormat + render_table
│   │       │   ├── output.rs  # JSON输出格式化
│   │       │   └── theme.rs   # 可定制主题
│   │       └── utils/
│   │           ├── date.rs     # parse_date(), parse_datetime()
│   │           └── validation.rs # validate_*() + 21 个单元测试
│   ├── i-rs-{name}...          # 70个 CLI 工具
├── docs/                       # VitePress 文档站点
│   └── .vitepress/
│       └── config.ts           # 侧边栏配置
├── skills/                     # AI 技能文档 (70个)
├── scripts/                    # 辅助脚本
├── .github/workflows/
│   ├── release.yml             # cargo-dist 自动发布
│   └── check.yml               # CI: cargo check + clippy + fmt
├── deny.toml                   # cargo-deny 配置
├── rust-toolchain.toml         # 工具链固定
├── Cargo.toml                  # Workspace 配置
├── Cargo.lock                  # 依赖锁定 (已提交)
├── README.md
├── SPEC.md                     # 本规范文档
└── AGENTS.md                   # 开发规范 (AI)
```

## 2. 工具列表 (当前 70 个)

### 核心工具
| 工具 | 描述 | 特殊命令 |
|------|------|---------|
| i-rs-server | 服务器管理 | suggest |
| i-rs-password | 密码管理 | - |
| i-rs-bookmark | 书签管理 | - |
| i-rs-note | 笔记管理 | - |
| i-rs-todo | 待办管理 | done |
| i-rs-keys | API密钥管理 | - |
| i-rs-kv | 键值存储 | - |
| i-rs-deploy | 部署追踪 | rollback, stats |

### 健康追踪
| 工具 | 描述 | 特殊命令 |
|------|------|---------|
| i-rs-weight | 体重追踪 | chart, stats |
| i-rs-height | 身高追踪 | stats |
| i-rs-mood | 心情记录 | calendar |
| i-rs-sleep | 睡眠追踪 | stats |
| i-rs-water | 喝水记录 | - |
| i-rs-step | 步数记录 | - |
| i-rs-dose | 吃药记录 | - |
| i-rs-cycle | 经期记录 | - |
| i-rs-sit | 久坐提醒 | - |
| i-rs-allergy | 过敏记录 | - |
| i-rs-cal | 卡路里估算 | - |
| i-rs-fast | 断食记录 | - |
| i-rs-exercise | 运动记录 | stats |
| i-rs-run | 跑步记录 | plan, stats |
| i-rs-cycling | 骑行记录 | stats |
| i-rs-vision | 视力追踪 | stats |
| i-rs-habit | 习惯追踪 | checkin, streak |

### 提醒与到期
| 工具 | 描述 | 特殊命令 |
|------|------|---------|
| i-rs-domain | 域名到期 | - |
| i-rs-remind | 提醒管理 | done |
| i-rs-sub | 订阅追踪 | - |
| i-rs-bestby | 物品过期 | - |
| i-rs-tick | 时长记录 | - |
| i-rs-time | 时间追踪 | start, stop, report, stats |
| i-rs-event | 活动管理 | stats |
| i-rs-birthday | 生日提醒 | stats |

### 财务与数据
| 工具 | 描述 | 特殊命令 |
|------|------|---------|
| i-rs-ledger | 记账 | - |
| i-rs-recur | 固定支出 | - |
| i-rs-budget | 预算管理 | stats, expense |
| i-rs-invest | 投资追踪 | stats |
| i-rs-debt | 债务管理 | pay, stats |
| i-rs-invoice | 发票管理 | stats |
| i-rs-tax | 税务记录 | stats |
| i-rs-goal | 储蓄目标 | deposit, milestone, stats |

### 笔记与阅读
| 工具 | 描述 | 特殊命令 |
|------|------|---------|
| i-rs-note | 笔记管理 | - |
| i-rs-bookmark | 书签管理 | - |
| i-rs-article | 文章追踪 | stats |
| i-rs-read | 阅读进度 | stats |
| i-rs-quote | 名言收集 | - |
| i-rs-spark | 灵感记录 | - |
| i-rs-snippet | 代码片段 | - |
| i-rs-vocab | 单词学习 | quiz, stats |

### 饮食与生活
| 工具 | 描述 | 特殊命令 |
|------|------|---------|
| i-rs-meal | 餐食记录 | - |
| i-rs-pig | 猪瘾记录 | - |
| i-rs-grocery | 购物清单 | purchase, clear |
| i-rs-want | 愿望清单 | - |
| i-rs-gift | 礼物管理 | stats |
| i-rs-movie | 电影追踪 | stats |
| i-rs-podcast | 播客追踪 | stats |
| i-rs-contact | 联系人管理 | remind, stats |
| i-rs-car | 车辆管理 | fuel, maintain, stats |
| i-rs-project | 项目管理 | milestone, stats |

### 家居护理 (9个)
| 工具 | 描述 |
|------|------|
| i-rs-sheet | 换床单 |
| i-rs-toothbrush | 牙刷更换 |
| i-rs-towel | 毛巾更换 |
| i-rs-bed | 床垫更换 |
| i-rs-ac | 空调清洗 |
| i-rs-filter | 滤网清洗 |
| i-rs-purify | 净水器滤芯 |
| i-rs-appliance | 家电管理 |
| i-rs-plant | 植物养护 |

### 宠物护理 (4个)
| 工具 | 描述 |
|------|------|
| i-rs-feedpet | 宠物喂食 |
| i-rs-petbath | 宠物洗澡 |
| i-rs-walkdog | 遛狗记录 |
| i-rs-aqua | 鱼缸换水 |

## 3. i-rs-core 共享库

### 3.1 模块结构

```
i-rs-core/src/
├── lib.rs                    # 公共API导出
├── macro.rs                  # 宏: create_store!, skill_command!, exit_on_error!
├── storage/                  # 通用存储
│   └── mod.rs              # Storage<T>, filter_by_tag, HasTags
├── presentation/             # 输出格式化
│   ├── mod.rs              # print_error/success/header/warning + render_table
│   ├── output.rs           # output_list/output_item/output_error
│   └── theme.rs            # 可定制主题 (theme.json)
└── utils/
    ├── date.rs             # parse_date(), parse_datetime()
    └── validation.rs       # validate_name/url/weight/amount + 21 个单元测试
```

### 3.2 公共导出

```rust
// Storage
pub use i_rs_core::storage::{Storage, filter_by_tag, HasTags};

// Presentation
pub use i_rs_core::presentation::{
    print_error, print_header, print_success, print_warning, println_dimmed,
    render_table, OutputFormat, Theme,
};
pub use i_rs_core::presentation::output::{output_list, output_item, output_error};

// Utils
pub use i_rs_core::utils::date::{parse_date, parse_datetime};
pub use i_rs_core::utils::validation::{
    validate_name, validate_url, validate_weight, validate_amount, ValidationError,
};

// Macros (used via i_rs_core::create_store! etc.)
// - create_store!(XxxStore, "xxx")
// - skill_command!("i-rs-xxx")
// - exit_on_error!(result, json)
```

### 3.3 宏速查表

| 宏 | 作用 | 使用位置 |
|----|------|----------|
| `create_store!(Type, "name")` | 生成 load_store/save_store | `storage/mod.rs` |
| `skill_command!("crate")` | 生成 SkillCommand + handle_skill | `commands/skill.rs` |
| `exit_on_error!(result, json)` | 统一错误处理 + JSON 输出 | `main.rs` |

## 4. Crate 开发流程 (清单)

### 4.1 创建新工具步骤

1. **创建目录结构**
```bash
mkdir -p crates/i-rs-{name}/src/{models,storage,commands,presentation}
mkdir -p docs/crates/i-rs-{name}
mkdir -p skills/i-rs-{name}
```

2. **创建 Cargo.toml**
```toml
[package]
name = "i-rs-{name}"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
i-rs-core = { path = "../i-rs-core" }
clap.workspace = true
anyhow.workspace = true
serde.workspace = true
tabled.workspace = true
owo-colors.workspace = true
chrono.workspace = true
uuid.workspace = true          # 如需UUID
keyring.workspace = true       # 如需密码存储
keyring-core.workspace = true
```

3. **实现源代码**
- `src/models/mod.rs` - 数据结构 (BTreeMap 存储) + Row 结构 (Tabled) + Store 结构 + CRUD 方法
- `src/storage/mod.rs` - 使用 `create_store!` 宏
- `src/presentation/mod.rs` - 使用 `render_table()`
- `src/commands/mod.rs` - 命令导出
- `src/commands/add.rs` - 添加命令
- `src/commands/delete.rs` - 删除命令
- `src/commands/get.rs` - 获取命令
- `src/commands/list.rs` - 列表命令
- `src/commands/update.rs` - 更新命令
- `src/commands/example.rs` - 示例命令
- `src/commands/skill.rs` - 使用 `skill_command!` 宏
- `src/main.rs` - CLI入口，使用 `exit_on_error!` 宏

4. **创建 README.md** (必需!)

5. **创建文档** (必需!)
- `docs/crates/i-rs-{name}/index.md` - 概述
- `docs/crates/i-rs-{name}/usage.md` - 使用说明
- `docs/crates/i-rs-{name}/examples.md` - 示例
- `docs/crates/i-rs-{name}/test.md` - 测试数据

6. **创建 Skills** (必需!)
- `skills/i-rs-{name}/SKILL.md` - AI技能文档 (包含 YAML frontmatter)

7. **更新 Workspace Cargo.toml**
在根目录 `Cargo.toml` 的 `members` 中添加 `"crates/i-rs-{name}"`

8. **更新 VitePress 配置** (必需!)
在 `docs/.vitepress/config.ts` 侧边栏添加条目

9. **编译验证**
```bash
cargo build -p i-rs-{name}
cargo check
```

### 4.2 文档完整性检查清单

- [ ] `crates/i-rs-{name}/README.md` 存在
- [ ] `docs/crates/i-rs-{name}/index.md` 存在
- [ ] `docs/crates/i-rs-{name}/usage.md` 存在
- [ ] `docs/crates/i-rs-{name}/examples.md` 存在
- [ ] `docs/crates/i-rs-{name}/test.md` 存在
- [ ] `skills/i-rs-{name}/SKILL.md` 存在
- [ ] `docs/.vitepress/config.ts` 包含侧边栏条目
- [ ] `Cargo.toml` workspace 包含此 crate

## 5. 数据模型规范

### 5.1 实体结构 (命名实体)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub name: String,                    // 唯一标识
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
```

### 5.2 Store 规范

- **存储结构**: 必须使用 `BTreeMap<String, Entity>` (统一 BTreeMap, 不用 HashMap)
- **CRUD 方法命名**: 统一为 `add_entry`, `remove_entry`, `get_entry`, `get_entry_mut`
- **主键类型**: String (name) 或 UUID

### 5.3 敏感字段

```rust
#[serde(skip)]
pub password: Option<String>,
```

### 5.4 时间计算

```rust
pub fn days_until(&self) -> i64 {
    (self.event_date - Utc::now()).num_days()
}
```

## 6. CLI 设计规范

### 6.1 命令结构

```rust
#[derive(Parser, Debug)]
#[command(name = "i-rs-{name}")]
#[command(about = "Tool description", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    json: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Add { name: String, #[arg(short, long)] tags: Vec<String> },
    Delete { name: String },
    List { #[arg(short, long)] tag: Option<String> },
    Get { name: String },
    Update { name: String, #[arg(short, long)] tags: Vec<String> },
    Example {},
    Skill { sub: Option<String> },
}
```

### 6.2 main.rs 模板

```rust
fn main() {
    let cli = Cli::parse();
    let format = if cli.json { OutputFormat::Json } else { OutputFormat::Table };
    i_rs_core::exit_on_error!(run(cli.command, format), cli.json);
}
```

### 6.3 storage/mod.rs 模板

```rust
use crate::models::XxxStore;
i_rs_core::create_store!(XxxStore, "xxx");
```

`create_store!` 宏会生成以下函数：
- `load_store()` — 从磁盘加载数据
- `save_store()` — 保存数据到磁盘
- `export_data()` — 导出全部数据为 JSON 字符串
- `import_data(input)` — 从 JSON 字符串导入数据
- `clear_data()` — 清空所有数据（重置为默认值）

### 6.4 commands/data.rs 模板

```rust
use clap::Subcommand;
use std::io::Read;

#[derive(Subcommand, Debug, Clone)]
pub enum DataCommand {
    Export,
    Import { file: Option<String> },
    Clear,
}

pub fn handle(command: &DataCommand) -> anyhow::Result<()> {
    match command {
        DataCommand::Export => { ... },
        DataCommand::Import { file } => { ... },
        DataCommand::Clear => { ... },
    }
}
```

### 6.5 commands/skill.rs 模板

```rust
i_rs_core::skill_command!("i-rs-xxx");
```

### 6.6 Data 命令（通用子命令）

每个 crate 统一支持:

```bash
# 导出全部数据为 JSON
i-rs-xxx data export

# 从文件或 stdin 导入数据（覆盖式导入）
i-rs-xxx data import < backup.json
i-rs-xxx data import /path/to/file.json

# 清空所有数据
i-rs-xxx data clear
```

## 7. JSON输出规范

```json
{
  "success": true,
  "data": {...},
  "meta": { "count": 10, "filter": "work" }
}
```

错误响应:
```json
{
  "success": false,
  "error": {
    "code": "NOT_FOUND",
    "message": "Entry 'xxx' not found"
  }
}
```

## 8. 存储规范

- **密码**: 必须存储在 OS keychain 中
- **数据文件**: `~/.config/i-rs/{name}.json`
- **自定义路径**: `CONFIG_DIR` 环境变量
- **存储结构**: 统一使用 `BTreeMap` (不用 HashMap)
- **CRUD 方法**: 统一为 `add_entry`, `remove_entry`, `get_entry`, `get_entry_mut`

## 9. 输入验证

```rust
use i_rs_core::{validate_name, validate_url, validate_weight, validate_amount};

if let Err(e) = validate_name(&name) {
    print_error(&e.message);
    anyhow::bail!("{}", e.message);
}
```

### 验证规则

| 函数 | 规则 |
|------|------|
| `validate_name` | 非空, ≤100字符, 无 `/ \ : * ? " < > \|` |
| `validate_url` | 非空, 以 `http://` 或 `https://` 开头, ≤2000字符 |
| `validate_weight` | > 0, ≤1000 kg |
| `validate_amount` | > 0, ≤10亿 |

## 10. Bug 预防

### Store 加载模式 (正确)
```rust
let mut store = storage::load_store()?;
// 操作 store ...
storage::save_store(&store)?;
```

### Store 加载模式 (错误 - 双重加载!)
```rust
let store = storage::load_store()?;
// ...
let mut store = storage::load_store()?;  // BUG: 重复加载!
```

## 11. Workspace 依赖

```toml
[workspace.dependencies]
clap = { version = "4.5", features = ["derive"] }
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
dirs = "6.0.0"
keyring = "4.0.1"
keyring-core = "1.0.0"
tabled = { version = "0.20.0", features = ["ansi"] }
owo-colors = "4.3.0"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["v4"] }
```

### 各 crate Cargo.toml 注意事项
- **不要**添加 `dirs` (通过 i-rs-core 间接使用)
- **不要**添加 `serde_json` (仅当直接使用 serde_json::json! 时)
- **不要**添加 `tokio` 或 `reqwest` (当前无 crate 使用 async)
- **uuid** 按需添加 (仅当使用 `Uuid::new_v4()`)

## 12. 发布流程

```bash
# 更新版本号 (workspace.package.version in Cargo.toml)
git tag v0.0.x
git push origin v0.0.x
```

CI (cargo-dist) auto-builds 并发布到:
- GitHub Releases
- npm (`@i-rs/i-rs-*`)
- Homebrew (`i-rs/homebrew-tap/i-rs-*`)

## 13. 构建配置

```toml
[profile.release]
lto = "thin"
strip = true
codegen-units = 1

[profile.dist]
inherits = "release"
```

## 14. CI / 质量保障

- `check.yml` — push/PR 时运行 `cargo check` + `clippy` + `fmt`
- `release.yml` — tag 推送时 cargo-dist 发布
- `deny.toml` — cargo-deny 许可证/安全审计
- i-rs-core 有 21 个单元测试覆盖 validation 和 date 模块

## 15. 重要文件

- `SPEC.md` — 项目规范 (中文)
- `AGENTS.md` — AI 开发工作流
- `Cargo.toml` — Workspace 配置
- `rust-toolchain.toml` — Rust 工具链固定
- `deny.toml` — 依赖审计配置
- `docs/.vitepress/config.ts` — 文档侧边栏
- `.github/workflows/` — CI/CD 配置
