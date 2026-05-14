# i-rs CLI 项目规范

## 1. 项目结构

```
i-rs-clis/
├── crates/                    # 所有 CLI 工具
│   └── i-rs-{name}/          # 每个工具一个 crate
│       ├── src/
│       │   ├── main.rs        # 入口文件 (CLI解析 + --json全局标志)
│       │   ├── commands/      # 命令处理模块
│       │   │   ├── mod.rs
│       │   │   ├── add.rs
│       │   │   ├── delete.rs
│       │   │   ├── get.rs
│       │   │   ├── list.rs
│       │   │   ├── update.rs
│       │   │   ├── example.rs  # 示例命令
│       │   │   ├── skill.rs   # 技能命令
│       │   │   └── {special}.rs  # 可选特殊命令 (done, suggest 等)
│       │   ├── models/       # 数据模型
│       │   │   └── mod.rs
│       │   ├── storage/      # 存储和密钥链
│       │   │   └── mod.rs
│       │   └── presentation/  # 输出展示
│       │       ├── mod.rs
│       │       └── output.rs  # JSON输出格式化
│       ├── Cargo.toml
│       └── README.md
├── docs/                     # VitePress 文档站点
│   ├── index.md                  # 首页
│   ├── guide/
│   │   └── getting-started.md    # 快速入门
│   ├── crates/
│   │   └── i-rs-{name}/
│   │       ├── index.md          # 工具概览 (overview)
│   │       ├── usage.md         # 命令参考
│   │       ├── examples.md      # 使用示例
│   │       └── test.md          # 测试记录
│   └── .vitepress/
│       └── config.ts             # VitePress 侧边栏配置
├── skills/                   # AI 技能文档
│   └── i-rs-{name}/
│       └── SKILL.md
├── scripts/                  # 发布脚本
├── Cargo.toml                # Workspace 配置
├── README.md                 # 项目总览
├── AGENTS.md                 # 开发规范 (AI)
└── SPEC.md                  # 本规范文档
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

## 3. Crate 开发流程

### 3.1 创建新工具步骤

1. **创建目录结构**
```bash
mkdir -p crates/i-rs-{name}/src/{models,storage,commands,presentation}
mkdir -p docs/crates/i-rs-{name}
mkdir -p skills/i-rs-{name}
```

2. **创建 Cargo.toml** (使用 workspace 依赖)
```toml
[package]
name = "i-rs-{name}"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
description = "CLI tool description"

[dependencies]
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
- `storage/mod.rs` - 数据持久化 + keyring
- `presentation/mod.rs` - 表格格式化 + 颜色
- `presentation/output.rs` - JSON输出格式化
- `commands/mod.rs` - 命令路由 + 导出example/skill
- `main.rs` - CLI解析 + --json全局标志

4. **实现命令文件**
- `commands/add.rs` - 添加命令
- `commands/delete.rs` - 删除命令
- `commands/get.rs` - 获取命令 (支持JSON输出)
- `commands/list.rs` - 列表命令 (支持JSON输出)
- `commands/update.rs` - 更新命令
- `commands/example.rs` - 示例命令
- `commands/skill.rs` - 技能命令
- `commands/{special}.rs` - 特殊命令(如done, suggest)

5. **更新 Workspace 配置**
```toml
# Cargo.toml
[workspace]
members = [
    ...
    "crates/i-rs-{name}",
]
```

6. **更新 VitePress 配置**
```typescript
// docs/.vitepress/config.ts
{
  text: 'i-rs-{name}',
  collapsed: true,
  items: [
    { text: 'Overview', link: '/crates/i-rs-{name}/' },
    { text: 'Usage', link: '/crates/i-rs-{name}/usage' },
    { text: 'Examples', link: '/crates/i-rs-{name}/examples' },
    { text: 'Test', link: '/crates/i-rs-{name}/test' }
  ]
}
```

7. **编译验证**
```bash
cargo build -p i-rs-{name}
```

### 3.2 模块职责

| 模块 | 职责 |
|------|------|
| `models/` | 数据结构定义（Struct、Serialize/Deserialize） |
| `storage/` | 数据持久化、keyring 密码存储 |
| `commands/` | CLI 命令处理器（add、delete、get、list、update、example、skill） |
| `presentation/` | 输出格式化（表格、颜色、图表、JSON） |
| `main.rs` | CLI 解析、命令分发、全局 --json 标志 |

## 4. 数据模型规范

### 4.1 实体结构

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

### 4.2 表格行结构

```rust
#[derive(Tabled)]
pub struct EntityRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "CREATED")]
    created_at: String,
    #[tabled(rename = "UPDATED")]
    updated_at: String,
    // ... 其他列
}

impl EntityRow {
    pub fn from_entity(entity: &Entity) -> Self {
        Self {
            name: entity.name.clone(),
            created_at: entity.created_at.format("%Y-%m-%d %H:%M").to_string(),
            updated_at: entity.updated_at.format("%Y-%m-%d %H:%M").to_string(),
            // ...
        }
    }
}
```

### 4.3 时间计算 (用于 domain/remind/weight/mood)

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

## 5. CLI 设计规范

### 5.1 命令结构

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

### 5.2 短选项命名

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

### 5.3 日期解析

```rust
fn parse_date(date_str: &str) -> Result<NaiveDate> {
    let formats = ["%Y-%m-%d", "%Y/%m/%d", "%d-%m-%Y", "%d/%m/%Y"];
    for format in &formats {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, format) {
            return Ok(date);
        }
    }
    Err(anyhow::anyhow!("Invalid date format: {}. Use YYYY-MM-DD", date_str))
}
```

## 6. JSON输出规范

### 6.1 输出模块 (presentation/output.rs)

```rust
use serde::Serialize;

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Table,
    Json,
}

#[derive(Debug, Serialize)]
pub struct ListResponse<T: Serialize> {
    pub success: bool,
    pub data: Vec<T>,
    pub meta: ListMeta,
}

#[derive(Debug, Serialize)]
pub struct ListMeta {
    pub count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct ItemResponse<T: Serialize> {
    pub success: bool,
    pub data: T,
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: ErrorDetail,
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
}

pub fn output_list<T: Serialize + Clone>(items: &[T], count: usize, filter: Option<&str>, format: OutputFormat) -> String {
    match format {
        OutputFormat::Json => {
            let response = ListResponse {
                success: true,
                data: items.to_vec(),
                meta: ListMeta { count, filter: filter.map(String::from) },
            };
            serde_json::to_string_pretty(&response).unwrap_or_else(|_| r#"{"success":false,"error":{"code":"SERIALIZE_ERROR","message":"Failed to serialize"}}"#.to_string())
        }
        OutputFormat::Table => {
            serde_json::to_string(items).unwrap_or_default()
        }
    }
}

pub fn output_item<T: Serialize>(item: &T, format: OutputFormat) -> String {
    // 类似实现...
}

pub fn output_error(message: &str, code: &str, format: OutputFormat) -> String {
    // 类似实现...
}
```

### 6.2 JSON响应格式

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

### 6.3 命令处理示例

```rust
use crate::presentation::output::{output_list, output_item, output_error, OutputFormat};

pub fn handle_list(tag: Option<String>, format: OutputFormat) -> Result<()> {
    let store = storage::load_store()?;
    let items: Vec<&Entity> = storage::filter_by_tag(&store, tag.as_deref());

    if matches!(format, OutputFormat::Json) {
        // JSON输出逻辑
        let items: Vec<ListItem> = items.iter().map(|e| ListItem {
            name: e.name.clone(),
            // ...
        }).collect();
        println!("{}", output_list(&items, items.len(), tag.as_deref(), format));
        return Ok(());
    }

    // Table输出逻辑 (默认)
    let table = format_table(&items);
    println!("\n{}", table);
    Ok(())
}
```

## 7. 存储规范

### 7.1 密码存储

**密码必须存储在 OS keychain 中，绝不存储在配置文件中。**

```rust
const SERVICE_NAME: &str = "i-rs-{name}";

pub fn store_password(name: &str, password: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, name)?;
    entry.set_password(password)?;
    Ok(())
}
```

### 7.2 数据文件

- 位置：`~/.config/i-rs/{name}.json`
- 覆盖方式：`CONFIG_DIR` 环境变量可自定义路径

## 8. 表格展示规范

### 8.1 颜色配置

```rust
Table::new(&rows)
    .with(Style::modern_rounded())
    .modify(Segment::all(), BorderColor::filled(Color::FG_CYAN))
    .with(Colorization::exact([Color::FG_CYAN | Color::BOLD], Rows::first()))
    .with(Colorization::exact([Color::FG_GREEN], Rows::new(1..)))
    .to_string()
```

### 8.2 颜色含义

| 元素 | 颜色 |
|------|------|
| 边框 | 青色 (FG_CYAN) |
| 表头 | 青色 + 粗体 |
| 数据行 | 绿色 (FG_GREEN) |

## 9. 全局命令规范

### 9.1 example 命令

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

### 9.2 skill 命令

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

## 10. 文档规范

### 10.1 VitePress 配置

侧边栏采用折叠菜单，每个 crate 包含 4 个子页面：

```typescript
// docs/.vitepress/config.ts
{
  text: 'i-rs-{name}',
  collapsed: true,
  items: [
    { text: 'Overview', link: '/crates/i-rs-{name}/' },
    { text: 'Usage', link: '/crates/i-rs-{name}/usage' },
    { text: 'Examples', link: '/crates/i-rs-{name}/examples' },
    { text: 'Test', link: '/crates/i-rs-{name}/test' }
  ]
}
```

### 10.2 文档内容要求

| 文件 | 内容要求 |
|------|---------|
| `index.md` | 概述、Quick Start、安装命令、特性列表、mood levels/data storage |
| `usage.md` | 详细命令参考，所有选项说明 |
| `examples.md` | 丰富示例：基础操作、实际场景、脚本集成 |
| `test.md` | 测试记录：正常流程、错误处理 |

### 10.3 skills/{name}/SKILL.md

AI 技能文档，供 AI 助手理解工具用途和调用方式：

```markdown
---
name: "i-rs-{name}"
description: "工具描述。当用户需要...时使用。"
---

# i-rs-{name}
[简短描述]
## Storage
## Commands
### add
### list
### get
### update
### delete
## Examples
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
```

## 12. 错误处理

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

## 13. 代码风格

- 使用 `snake_case` 命名变量和函数
- 使用 `PascalCase` 命名结构体和枚举
- 使用 `camelCase` 命名 CLI 参数（clap 自动转换）
- 避免使用 `unwrap()`，使用 `?` 操作符
- 敏感字段添加 `#[allow(dead_code)]`
- 未使用的导出函数添加 `#[allow(dead_code)]`
- output.rs中未使用的结构体和函数添加 `#[allow(dead_code)]`