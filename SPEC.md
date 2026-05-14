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
│   ├── i-rs-password/         # 密码管理
│   ├── i-rs-bookmark/         # 书签管理
│   ├── i-rs-note/             # 笔记管理
│   ├── i-rs-domain/           # 域名管理
│   ├── i-rs-remind/           # 提醒管理
│   ├── i-rs-weight/           # 体重追踪
│   ├── i-rs-mood/             # 心情记录
│   └── i-rs-todo/             # 待办管理
├── docs/                      # VitePress 文档站点
├── skills/                    # AI 技能文档
├── Cargo.toml                 # Workspace 配置
├── README.md
├── SPEC.md                   # 本规范文档
└── AGENTS.md                  # 开发规范 (AI)
```

## 2. 工具列表 (当前 9 个)

| 工具 | 描述 | 特殊命令 |
|------|------|---------|
| i-rs-server | 服务器管理 | suggest |
| i-rs-password | 密码管理 | - |
| i-rs-bookmark | 书签管理 | - |
| i-rs-note | 笔记管理 | - |
| i-rs-domain | 域名管理 | - |
| i-rs-remind | 提醒管理 | done |
| i-rs-weight | 体重追踪 | chart, stats |
| i-rs-mood | 心情记录 | calendar |
| i-rs-todo | 待办管理 | done |

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

## 4. Crate 开发流程

### 4.1 创建新工具步骤

1. **创建目录结构**
```bash
mkdir -p crates/i-rs-{name}/src/{models,storage,commands,presentation}
mkdir -p docs/crates/i-rs-{name}
mkdir -p skills/i-rs-{name}
```

2. **创建 Cargo.toml** (依赖 i-rs-core)
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
```

3. **实现 6 个核心模块**
- `models/mod.rs` - 数据结构 + 表格行结构
- `storage/mod.rs` - 数据持久化 (使用 i-rs-core Storage) + keyring
- `presentation/mod.rs` - 表格格式化 + 颜色 + 打印函数
- `commands/mod.rs` - 命令路由 + 导出example/skill
- `commands/*.rs` - 命令处理器
- `main.rs` - CLI解析 + --json全局标志

4. **更新 Workspace 配置**
```toml
# Cargo.toml
[workspace]
members = [
    "crates/i-rs-core",
    ...
    "crates/i-rs-{name}",
]
```

5. **编译验证**
```bash
cargo build -p i-rs-{name}
cargo check
```

### 4.2 模块职责

| 模块 | 职责 |
|------|------|
| `models/` | 数据结构定义（Struct、Serialize/Deserialize） |
| `storage/` | 数据持久化（使用i-rs-core Storage）、keyring密码存储 |
| `commands/` | CLI 命令处理器（add、delete、get、list、update、example、skill） |
| `presentation/` | 输出格式化（表格、颜色、图表）、使用 i-rs-core 的 print/output 函数 |
| `main.rs` | CLI 解析、命令分发、全局 --json 标志 |

## 5. 数据模型规范

### 5.1 实体结构

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub name: String,                    // 唯一标识
    // 业务字段...
    #[serde(skip)]                       // 跳过序列化（密码等敏感数据）
    #[allow(dead_code)]
    pub password: Option<String>,
    #[serde(default)]                    // 默认空数组
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
```

### 5.2 表格行结构

```rust
#[derive(Tabled)]
pub struct EntityRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "CREATED")]
    created_at: String,
    #[tabled(rename = "UPDATED")]
    updated_at: String,
}

impl EntityRow {
    pub fn from_entity(entity: &Entity) -> Self {
        Self {
            name: entity.name.clone(),
            created_at: entity.created_at.format("%Y-%m-%d %H:%M").to_string(),
            updated_at: entity.updated_at.format("%Y-%m-%d %H:%M").to_string(),
        }
    }
}
```

### 5.3 时间计算 (用于 domain/remind/weight/mood)

```rust
// 距离到期/事件天数
pub fn days_until(&self) -> i64 {
    (self.event_date - Utc::now()).num_days()
}

// 是否已过期/已过去
pub fn is_past(&self) -> bool {
    self.days_until() < 0
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
    json: bool,  // 全局JSON输出标志
}

#[derive(Subcommand, Debug)]
enum Commands {
    Add {
        #[arg(value_name = "NAME")]
        name: String,
        // 其他必需参数
        #[arg(short, long)]
        optional: Option<String>,
        #[arg(short, long)]
        tags: Vec<String>,           // 可重复参数
        #[arg(short, long)]
        remark: Vec<String>,
    },
    Delete { name: String },
    List {
        #[arg(short, long)]
        tag: Option<String>,
    },
    Get {
        name: String,
        #[arg(short = 's', long)]
        show_password: bool,
    },
    Update {
        name: String,
        // 可选更新字段
    },
    Example {},
    Skill {
        #[arg(value_name = "SUB_COMMAND")]
        sub: Option<String>,
    },
}
```

### 6.2 短选项命名

| 选项 | 短选项 | 说明 |
|------|--------|------|
| `--user` | `-u` | 用户名 |
| `--password` | `-p` | 密码 |
| `--tag` | `-t` | 标签 |
| `--remark` | `-r` | 备注 |
| `--show-password` | `-s` | 显示密码 |
| `--days` | `-d` | 天数 |
| `--chart` | `-c` | 图表 |
| `--stats` | `-s` | 统计 |
| `--calendar` | `-c` | 日历 |
| `--content` | 无 | 内容行 |
| `--mood` | `-m` | 心情 |

## 7. JSON输出规范

JSON输出功能已移至 i-rs-core 的 `presentation/output.rs`。

### 7.1 JSON响应格式

**List Response (list命令):**
```json
{
  "success": true,
  "data": [...],
  "meta": {
    "count": 10,
    "filter": "work"
  }
}
```

**Item Response (get命令):**
```json
{
  "success": true,
  "data": {...}
}
```

**Error Response:**
```json
{
  "success": false,
  "error": {
    "code": "NOT_FOUND",
    "message": "Entry 'xxx' not found"
  }
}
```

### 7.2 命令处理示例

```rust
// presentation/mod.rs
pub use i_rs_core::presentation::{print_error, print_header, print_success, print_warning, OutputFormat};
pub use i_rs_core::presentation::output::{output_list, output_item, output_error};

// commands/list.rs
pub fn handle_list(tag: Option<String>, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;
    let items: Vec<&Entity> = storage::filter_by_tag(&store, tag.as_deref());

    if matches!(format, OutputFormat::Json) {
        let list_items: Vec<ListItem> = items.iter().map(|e| ListItem {
            name: e.name.clone(),
            // ...
        }).collect();
        println!("{}", output_list(&list_items, list_items.len(), tag.as_deref(), format));
        return Ok(());
    }

    // Table输出逻辑 (默认)
    let table = format_table(&items);
    println!("\n{}", table);
    Ok(())
}
```

## 8. 存储规范

### 8.1 密码存储

**密码必须存储在 OS keychain 中，绝不存储在配置文件中。**

```rust
const SERVICE_NAME: &str = "i-rs-{name}";

pub fn store_password(name: &str, password: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, name)?;
    entry.set_password(password)?;
    Ok(())
}
```

### 8.2 数据文件

- 位置：`~/.config/i-rs/{name}.json`
- 覆盖方式：`CONFIG_DIR` 环境变量可自定义路径

## 9. 输入验证

使用 i-rs-core 的验证函数：

```rust
use i_rs_core::{validate_name, validate_url, validate_weight, ValidationError};

// 在 add/update 命令中
if let Err(e) = validate_name(&name) {
    print_error(&e.message);
    anyhow::bail!("{}", e.message);
}
```

### 验证规则

| 函数 | 规则 |
|------|------|
| `validate_name` | 非空、≤100字符、无 `/ \ : * ? " < > \|` |
| `validate_url` | 非空、以 `http://` 或 `https://` 开头、≤2000字符 |
| `validate_weight` | > 0、≤1000 kg |

## 10. Bug 预防

### 10.1 Store加载模式 (正确)

```rust
// 正确：加载一次，使用可变引用
let mut store = storage::load_store()?;

if store.entries.contains_key(&name) {
    print_error(&format!("Entry '{}' already exists", name));
    anyhow::bail!("Entry '{}' already exists", name);
}

// ... 创建实体 ...

storage::add_entry(&mut store, entry);
storage::save_store(&store)?;
```

### 10.2 Store加载模式 (错误 - BUG!)

```rust
// 错误：双重加载 - 浪费I/O并导致bug
let store = storage::load_store()?;
if store.entries.contains_key(&name) { ... }

// ... 稍后 ...

let mut store = storage::load_store()?;  // BUG: 重新加载!
storage::add_entry(&mut store, entry);
```

## 11. 表格展示规范

### 11.1 颜色配置

```rust
// presentation/mod.rs
Table::new(&rows)
    .with(Style::modern_rounded())
    .modify(Segment::all(), BorderColor::filled(Color::FG_CYAN))
    .with(Colorization::exact([Color::FG_CYAN | Color::BOLD], Rows::first()))
    .with(Colorization::exact([Color::FG_GREEN], Rows::new(1..)))
    .to_string()
```

### 11.2 颜色含义

| 元素 | 颜色 |
|------|------|
| 边框 | 青色 (FG_CYAN) |
| 表头 | 青色 + 粗体 |
| 数据行 | 绿色 (FG_GREEN) |

## 12. 全局命令规范

### 12.1 example 命令

展示使用示例，帮助AI和用户快速理解CLI。

```rust
// commands/example.rs
use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-{name} Examples".bold().cyan());
    println!();
    println!("{}", "Add Entry:".bold().green());
    println!("  i-rs-{name} add <name> <value> --tag work");
    // 更多示例...
}
```

### 12.2 skill 命令

集成AI技能文档到CLI本身。

```rust
// commands/skill.rs
use clap::Parser;

#[derive(Parser, Debug)]
pub enum SkillCommand {
    Summary,
    Content,
    Raw,
}

const SKILL_SUMMARY: &str = r#"Tool description..."#;
const SKILL_CONTENT: &str = r#"Commands: add, list, get..."#;
const SKILL_RAW: &str = r#"--- name: "i-rs-{name}" ..."#;

pub fn handle_skill(which: Option<SkillCommand>) {
    match which {
        Some(SkillCommand::Summary) => println!("{}", SKILL_SUMMARY),
        Some(SkillCommand::Content) => println!("{}", SKILL_CONTENT),
        Some(SkillCommand::Raw) | None => println!("{}", SKILL_RAW),
    }
}
```

## 13. 错误处理

使用 `anyhow` 进行错误处理，主函数返回 `anyhow::Result<()>`：

```rust
fn main() {
    let cli = Cli::parse();
    let format = if cli.json {
        OutputFormat::Json
    } else {
        OutputFormat::Table
    };

    if let Err(e) = run(cli.command, format) {
        if cli.json {
            println!("{}", serde_json::json!({
                "success": false,
                "error": { "code": "UNKNOWN", "message": e.to_string() }
            }));
        } else {
            eprintln!("Error: {}", e);
        }
        std::process::exit(1);
    }
}

fn run(command: Commands, format: OutputFormat) -> anyhow::Result<()> {
    // 业务逻辑
    Ok(())
}
```

## 14. 代码风格

- 使用 `snake_case` 命名变量和函数
- 使用 `PascalCase` 命名结构体和枚举
- 使用 `camelCase` 命名 CLI 参数（clap 自动转换）
- 避免使用 `unwrap()`，使用 `?` 操作符
- 敏感字段添加 `#[allow(dead_code)]`
- 未使用方法添加 `#[allow(dead_code)]`

## 15. Workspace 依赖

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
```