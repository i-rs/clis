# i-rs CLI 项目规范

## 1. 项目结构

```
i-rs-clis/
├── crates/
│   ├── i-rs-core/              # 共享核心库
│   │   └── src/
│   │       ├── lib.rs          # 公共API导出
│   │       ├── storage/        # 通用存储 (Storage<T>)
│   │       ├── presentation/   # 输出格式化
│   │       │   ├── mod.rs     # print函数 + OutputFormat
│   │       │   └── output.rs  # JSON输出格式化
│   │       └── utils/
│   │           ├── date.rs     # parse_date()
│   │           └── validation.rs # validate_*()
│   ├── i-rs-server/            # 服务器管理
│   ├── i-rs-password/          # 密码管理
│   ├── i-rs-bookmark/          # 书签管理
│   ├── i-rs-note/              # 笔记管理
│   ├── i-rs-domain/            # 域名管理
│   ├── i-rs-remind/            # 提醒管理
│   ├── i-rs-weight/            # 体重追踪
│   ├── i-rs-mood/              # 心情记录
│   ├── i-rs-todo/              # 待办管理
│   ├── i-rs-water/             # 喝水记录
│   ├── i-rs-step/              # 步数记录
│   ├── i-rs-dose/              # 吃药记录
│   ├── i-rs-cycle/             # 经期记录
│   ├── i-rs-sit/               # 久坐提醒
│   ├── i-rs-allergy/           # 过敏记录
│   ├── i-rs-cal/               # 卡路里估算
│   ├── i-rs-fast/              # 断食记录
│   ├── i-rs-sheet/             # 换床单记录
│   ├── i-rs-feedpet/           # 宠物喂食
│   ├── i-rs-petbath/           # 宠物洗澡
│   ├── i-rs-walkdog/           # 遛狗记录
│   ├── i-rs-aqua/              # 鱼缸换水
│   ├── i-rs-ac/                # 空调清洗
│   ├── i-rs-filter/            # 滤网清洗
│   ├── i-rs-purify/            # 净水器滤芯
│   ├── i-rs-toothbrush/        # 牙刷更换
│   ├── i-rs-towel/             # 毛巾更换
│   ├── i-rs-bed/               # 床垫更换
│   ├── i-rs-sub/               # 订阅追踪
│   ├── i-rs-bestby/            # 物品过期追踪
│   ├── i-rs-ledger/            # 记账
│   ├── i-rs-recur/              # 固定支出
│   ├── i-rs-kv/                # 键值存储
│   ├── i-rs-keys/              # API密钥管理
│   ├── i-rs-meal/              # 餐食记录
│   ├── i-rs-pig/               # 猪瘾记录
│   ├── i-rs-tick/              # 时长记录
│   ├── i-rs-spark/             # 灵感记录
│   └── i-rs-want/              # 愿望清单
├── docs/                       # VitePress 文档站点
│   └── .vitepress/
│       └── config.ts           # 侧边栏配置
├── skills/                     # AI 技能文档
├── scripts/                    # 辅助脚本
├── Cargo.toml                  # Workspace 配置
├── README.md
├── SPEC.md                     # 本规范文档
└── AGENTS.md                   # 开发规范 (AI)
```

## 2. 工具列表 (当前 39 个)

### 核心工具 (5个)
| 工具 | 描述 | 特殊命令 |
|------|------|---------|
| i-rs-server | 服务器管理 | suggest |
| i-rs-password | 密码管理 | - |
| i-rs-bookmark | 书签管理 | - |
| i-rs-note | 笔记管理 | - |
| i-rs-todo | 待办管理 | done |

### 健康追踪 (10个)
| 工具 | 描述 | 特殊命令 |
|------|------|---------|
| i-rs-weight | 体重追踪 | chart, stats |
| i-rs-mood | 心情记录 | calendar |
| i-rs-water | 喝水记录 | - |
| i-rs-step | 步数记录 | - |
| i-rs-dose | 吃药记录 | - |
| i-rs-cycle | 经期记录 | - |
| i-rs-sit | 久坐提醒 | - |
| i-rs-allergy | 过敏记录 | - |
| i-rs-cal | 卡路里估算 | - |
| i-rs-fast | 断食记录 | - |

### 提醒与到期 (4个)
| 工具 | 描述 | 特殊命令 |
|------|------|---------|
| i-rs-domain | 域名管理 | - |
| i-rs-remind | 提醒管理 | done |
| i-rs-sub | 订阅追踪 | - |
| i-rs-bestby | 物品过期追踪 | - |

### 财务与数据 (4个)
| 工具 | 描述 | 特殊命令 |
|------|------|---------|
| i-rs-ledger | 记账 | - |
| i-rs-recur | 固定支出 | - |
| i-rs-kv | 键值存储 | - |
| i-rs-keys | API密钥管理 | - |

### 饮食与生活 (2个)
| 工具 | 描述 | 特殊命令 |
|------|------|---------|
| i-rs-meal | 餐食记录 | - |
| i-rs-pig | 猪瘾记录 | - |

### 时间与效率 (3个)
| 工具 | 描述 | 特殊命令 |
|------|------|---------|
| i-rs-tick | 时长记录 | - |
| i-rs-spark | 灵感记录 | - |
| i-rs-want | 愿望清单 | - |

### 家居护理 (7个)
| 工具 | 描述 | 特殊命令 |
|------|------|---------|
| i-rs-sheet | 换床单记录 | - |
| i-rs-toothbrush | 牙刷更换 | - |
| i-rs-towel | 毛巾更换 | - |
| i-rs-bed | 床垫更换 | - |
| i-rs-ac | 空调清洗 | - |
| i-rs-filter | 滤网清洗 | - |
| i-rs-purify | 净水器滤芯 | - |

### 宠物护理 (4个)
| 工具 | 描述 | 特殊命令 |
|------|------|---------|
| i-rs-feedpet | 宠物喂食 | - |
| i-rs-petbath | 宠物洗澡 | - |
| i-rs-walkdog | 遛狗记录 | - |
| i-rs-aqua | 鱼缸换水 | - |

## 3. i-rs-core 共享库

### 3.1 模块结构

```
i-rs-core/src/
├── lib.rs                    # 公共API导出
├── storage/                  # 通用存储
│   └── mod.rs              # Storage<T>, filter_by_tag, HasTags
├── presentation/             # 输出格式化
│   ├── mod.rs              # print_error/success/header/warning + OutputFormat
│   └── output.rs           # output_list/output_item/output_error
└── utils/
    ├── date.rs             # parse_date()
    └── validation.rs       # validate_name/validate_url/validate_weight
```

### 3.2 公共导出

```rust
// Storage
pub use i_rs_core::storage::{Storage, filter_by_tag, HasTags};

// Presentation
pub use i_rs_core::presentation::{print_error, print_header, print_success, print_warning, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item, output_error};

// Utils
pub use i_rs_core::utils::parse_date;
pub use i_rs_core::utils::validation::{validate_name, validate_url, validate_weight, ValidationError};
```

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

[dependencies]
i-rs-core = { path = "../i-rs-core" }
clap.workspace = true
anyhow.workspace = true
serde.workspace = true
serde_json.workspace = true
dirs.workspace = true
keyring.workspace = true       # 如需密码存储
keyring-core.workspace = true
tabled.workspace = true
owo-colors.workspace = true
chrono.workspace = true
uuid.workspace = true          # 如需UUID
```

3. **实现源代码**
- `src/models/mod.rs` - 数据结构 + 表格行结构
- `src/storage/mod.rs` - 数据持久化
- `src/presentation/mod.rs` - 表格格式化
- `src/commands/mod.rs` - 命令导出
- `src/commands/add.rs` - 添加命令
- `src/commands/delete.rs` - 删除命令
- `src/commands/get.rs` - 获取命令
- `src/commands/list.rs` - 列表命令
- `src/commands/update.rs` - 更新命令
- `src/commands/example.rs` - 示例命令
- `src/commands/skill.rs` - 技能命令
- `src/main.rs` - CLI入口

4. **创建 README.md** (必需!)
```markdown
# i-rs-{name}

描述 CLI 工具。

## Features
- 特性1
- 特性2

## Quick Start
```bash
i-rs-{name} add ...
i-rs-{name} list
```

## License
MIT OR Apache-2.0
```

5. **创建文档** (必需!)
- `docs/crates/i-rs-{name}/index.md` - 概述
- `docs/crates/i-rs-{name}/usage.md` - 使用说明
- `docs/crates/i-rs-{name}/examples.md` - 示例
- `docs/crates/i-rs-{name}/test.md` - 测试数据

6. **创建 Skills** (必需!)
- `skills/i-rs-{name}/SKILL.md` - AI技能文档

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

### 5.1 实体结构

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

### 5.2 敏感字段

```rust
#[serde(skip)]
#[allow(dead_code)]
pub password: Option<String>,
```

### 5.3 时间计算

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

## 7. JSON输出规范

```json
{
  "success": true,
  "data": {...},
  "meta": { "count": 10, "filter": "work" }
}
```

## 8. 存储规范

- **密码**: 必须存储在 OS keychain 中
- **数据文件**: `~/.config/i-rs/{name}.json`
- **自定义路径**: `CONFIG_DIR` 环境变量

## 9. 输入验证

```rust
use i_rs_core::{validate_name, validate_url, validate_weight};

if let Err(e) = validate_name(&name) {
    print_error(&e.message);
    anyhow::bail!("{}", e.message);
}
```

## 10. Bug 预防

### 正确模式
```rust
let mut store = storage::load_store()?;
storage::add_entry(&mut store, entry);
storage::save_store(&store)?;
```

### 错误模式 (双重加载!)
```rust
let store = storage::load_store()?;
// ...
let mut store = storage::load_store()?; // BUG!
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
uuid = { version = "1.0", features = ["v4", "serde"] }
```