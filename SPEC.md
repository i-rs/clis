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
│       │   │   └── update.rs
│       │   ├── models/       # 数据模型
│       │   │   └── mod.rs
│       │   ├── storage/      # 存储和密钥链
│       │   │   └── mod.rs
│       │   └── presentation/ # 输出展示
│       │       └── mod.rs
│       ├── Cargo.toml
│       └── README.md
├── docs/                     # VitePress 文档站点
│   ├── index.md                  # 首页
│   ├── guide/
│   │   └── getting-started.md    # 快速入门
│   ├── crates/
│   │   └── i-rs-{name}/
│   │       ├── index.md          # 工具概览
│   │       ├── usage.md          # 命令参考
│   │       ├── examples.md       # 使用示例
│   │       └── test.md           # 测试记录
│   └── .vitepress/
│       └── config.ts             # VitePress 配置
├── skills/                   # AI 技能文档
│   └── i-rs-{name}/
│       └── SKILL.md
├── scripts/                  # 发布脚本
├── Cargo.toml                # Workspace 配置
└── README.md                 # 项目总览
```

## 2. Crate 规范

### 2.1 Cargo.toml 结构

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
keyring.workspace = true
keyring-core.workspace = true
tabled.workspace = true
owo-colors.workspace = true
chrono.workspace = true
```

### 2.2 模块职责

| 模块 | 职责 |
|------|------|
| `models/` | 数据结构定义（Struct、Serialize/Deserialize） |
| `storage/` | 数据持久化、keyring 密码存储 |
| `commands/` | CLI 命令处理器（add、delete、get、list、update） |
| `presentation/` | 输出格式化（表格、颜色） |
| `main.rs` | CLI 解析和命令分发 |

## 3. 数据模型规范

### 3.1 实体结构

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

### 3.2 表格行结构

```rust
#[derive(Tabled)]
pub struct EntityRow {
    #[tabled(rename = "NAME")]
    name: String,
    // ... 其他列
}

impl EntityRow {
    pub fn from_entity(entity: &Entity) -> Self {
        Self {
            name: entity.name.clone(),
            // ...
        }
    }
}
```

## 4. CLI 设计规范

### 4.1 命令结构

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
        show_sensitive: bool,
    },
    Update {
        name: String,
        // 可选更新字段
    },
}
```

### 4.2 短选项命名

| 选项 | 短选项 | 说明 |
|------|--------|------|
| `--user` | `-u` | 用户名 |
| `--password` | `-p` | 密码 |
| `--tag` | `-t` | 标签 |
| `--remark` | `-r` | 备注 |
| `--show-password` | `-s` | 显示密码 |

## 5. 存储规范

### 5.1 密码存储

**密码必须存储在 OS keychain 中，绝不存储在配置文件中。**

```rust
const SERVICE_NAME: &str = "i-rs-{name}";

pub fn store_password(name: &str, password: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, name)?;
    entry.set_password(password)?;
    Ok(())
}
```

### 5.2 数据文件

- 位置：`~/.config/i-rs/{name}.json`
- 覆盖方式：`CONFIG_DIR` 环境变量可自定义路径

## 6. 表格展示规范

### 6.1 颜色配置

```rust
Table::new(&rows)
    .with(Style::modern_rounded())
    .modify(Segment::all(), BorderColor::filled(Color::FG_CYAN))
    .with(Colorization::exact([Color::FG_CYAN | Color::BOLD], Rows::first()))
    .with(Colorization::exact([Color::FG_GREEN], Rows::new(1..)))
    .to_string()
```

### 6.2 颜色含义

| 元素 | 颜色 |
|------|------|
| 边框 | 青色 (FG_CYAN) |
| 表头 | 青色 + 粗体 |
| 数据行 | 绿色 (FG_GREEN) |

## 7. 文档规范

文档采用 VitePress 静态站点，位于 `docs/` 目录。

### 7.1 VitePress 配置 (`docs/.vitepress/config.ts`)

```typescript
import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'i-rs CLI Tools',
  description: 'Cross-platform CLI tools built with Rust',
  appearance: 'dark',
  themeConfig: {
    nav: [
      { text: 'Home', link: '/' },
      { text: 'Guide', link: '/guide/getting-started' },
      { text: 'Tools', link: '/crates/i-rs-server/' }
    ],
    sidebar: [
      {
        text: 'Guide',
        items: [
          { text: 'Getting Started', link: '/guide/getting-started' }
        ]
      },
      {
        text: 'CLI Tools',
        items: [
          { text: 'i-rs-{name}', link: '/crates/i-rs-{name}/' }
        ]
      }
    ],
    socialLinks: [
      { icon: 'github', link: 'https://github.com/i-rs/clis' }
    ]
  }
})
```

### 7.2 文档目录结构

每个 crate 在 `docs/crates/i-rs-{name}/` 下包含 4 个文件：

| 文件 | 用途 |
|------|------|
| `index.md` | 工具概览、Quick Start、安全说明、数据存储位置、子页面链接 |
| `usage.md` | 详细命令参考（所有命令及选项说明） |
| `examples.md` | 丰富使用示例（基础操作、实际场景、脚本集成、故障排查） |
| `test.md` | 测试记录（手动测试用例、安全测试、跨平台测试、性能基准） |

### 7.3 文档首页 (`docs/index.md`)

采用 VitePress home layout，包含 hero 区域和 feature 列表：

```yaml
---
layout: home
hero:
  name: i-rs CLI Tools
  text: Cross-platform CLI tools built with Rust
  tagline: <简短的项目说明>
  actions:
    - theme: brand
      text: Get Started
      link: /guide/getting-started
features:
  - title: <Feature 名称>
    details: <Feature 描述>
---
```

### 7.4 快速入门 (`docs/guide/getting-started.md`)

包含安装方式（npm/Homebrew/源码编译）、每个工具的 quick start 示例、
数据存储说明、安全说明、开发与发布指南。

### 7.5 README.md (crate 根目录)

每个 crate 的 README.md 与 `docs/crates/i-rs-{name}/index.md` 内容保持一致，
作为 GitHub 仓库直接浏览时的入口文档。

```markdown
# i-rs-{name}

工具描述

## Install
安装命令

## Security
安全说明（密码存储方式）

## Usage
命令使用说明

## Examples
使用示例

## Data Storage
数据存储位置

## Commands

- [Usage](./docs/crates/i-rs-{name}/usage.md) - 详细命令参考
- [Examples](./docs/crates/i-rs-{name}/examples.md) - 使用示例
- [Test](./docs/crates/i-rs-{name}/test.md) - 测试记录

## License
```

### 7.6 skills/{name}/SKILL.md

AI 技能文档，供 AI 助手理解工具用途和调用方式。

```markdown
---
name: "i-rs-{name}"
description: "工具描述。当用户需要...时使用。"
---

# i-rs-{name}

## Security
## Storage
## Commands
### add
### list
### get
### update
### delete
## Examples
```

## 8. 发布规范

### 8.1 版本号

遵循 Semantic Versioning：`0.0.x`

### 8.2 发布流程

```bash
# 1. 更新版本
git tag v0.0.x
git push origin v0.0.x

# 2. CI 自动构建并发布
# - GitHub Release
# - npm
# - Homebrew
```

### 8.3 npm 包名

格式：`@i-rs/i-rs-{name}`

### 8.4 Homebrew tap

格式：`i-rs/homebrew-tap/i-rs-{name}`

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
