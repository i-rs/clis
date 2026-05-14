# i-rs CLI 项目规范

## 1. 项目结构

```
i-rs-clis/
├── crates/                    # 所有 CLI 工具
│   └── i-rs-{name}/          # 每个工具一个 crate
│       ├── src/
│       │   ├── main.rs        # 入口文件
│       │   ├── commands/      # 命令处理模块
│       │   │   ├── mod.rs
│       │   │   ├── add.rs
│       │   │   ├── delete.rs
│       │   │   ├── get.rs
│       │   │   ├── list.rs
│       │   │   ├── update.rs
│       │   │   └── {special}.rs  # 可选特殊命令 (done, suggest 等)
│       │   ├── models/       # 数据模型
│       │   │   └── mod.rs
│       │   ├── storage/      # 存储和密钥链
│       │   │   └── mod.rs
│       │   └── presentation/  # 输出展示
│       │       └── mod.rs
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

3. **实现 5 个核心模块**
- `models/mod.rs` - 数据结构 + 表格行结构
- `storage/mod.rs` - 数据持久化 + keyring
- `presentation/mod.rs` - 表格格式化 + 颜色
- `commands/mod.rs` - 命令路由
- `main.rs` - CLI 解析

4. **更新 Workspace 配置**
```toml
# Cargo.toml
[workspace]
members = [
    ...
    "crates/i-rs-{name}",
]
```

5. **更新 VitePress 配置**
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

6. **编译验证**
```bash
cargo build -p i-rs-{name}
```

### 3.2 模块职责

| 模块 | 职责 |
|------|------|
| `models/` | 数据结构定义（Struct、Serialize/Deserialize） |
| `storage/` | 数据持久化、keyring 密码存储 |
| `commands/` | CLI 命令处理器（add、delete、get、list、update） |
| `presentation/` | 输出格式化（表格、颜色、图表） |
| `main.rs` | CLI 解析和命令分发 |

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

## 6. 存储规范

### 6.1 密码存储

**密码必须存储在 OS keychain 中，绝不存储在配置文件中。**

```rust
const SERVICE_NAME: &str = "i-rs-{name}";

pub fn store_password(name: &str, password: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, name)?;
    entry.set_password(password)?;
    Ok(())
}
```

### 6.2 数据文件

- 位置：`~/.config/i-rs/{name}.json`
- 覆盖方式：`CONFIG_DIR` 环境变量可自定义路径

## 7. 表格展示规范

### 7.1 颜色配置

```rust
Table::new(&rows)
    .with(Style::modern_rounded())
    .modify(Segment::all(), BorderColor::filled(Color::FG_CYAN))
    .with(Colorization::exact([Color::FG_CYAN | Color::BOLD], Rows::first()))
    .with(Colorization::exact([Color::FG_GREEN], Rows::new(1..)))
    .to_string()
```

### 7.2 颜色含义

| 元素 | 颜色 |
|------|------|
| 边框 | 青色 (FG_CYAN) |
| 表头 | 青色 + 粗体 |
| 数据行 | 绿色 (FG_GREEN) |

## 8. 文档规范

### 8.1 VitePress 配置

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

### 8.2 文档内容要求

| 文件 | 内容要求 |
|------|---------|
| `index.md` | 概述、Quick Start、安装命令、特性列表、mood levels/data storage |
| `usage.md` | 详细命令参考，所有选项说明 |
| `examples.md` | 丰富示例：基础操作、实际场景、脚本集成 |
| `test.md` | 测试记录：正常流程、错误处理 |

### 8.3 skills/{name}/SKILL.md

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

## 9. Workspace 依赖

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

## 10. 错误处理

使用 `anyhow` 进行错误处理，主函数返回 `anyhow::Result<()>`：

```rust
fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    // 业务逻辑
    Ok(())
}
```

## 11. 代码风格

- 使用 `snake_case` 命名变量和函数
- 使用 `PascalCase` 命名结构体和枚举
- 使用 `camelCase` 命名 CLI 参数（clap 自动转换）
- 避免使用 `unwrap()`，使用 `?` 操作符
- 敏感字段添加 `#[allow(dead_code)]`
- 导出函数添加 `#[allow(dead_code)]` 如有未使用警告
