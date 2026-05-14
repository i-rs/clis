# i-rs-read - 阅读进度追踪

i-rs-read 是一个轻量级的 CLI 工具，用于追踪和管理你的阅读进度。无论你是在读技术书籍、小说还是杂志，i-rs-read 都能帮助你记录阅读状态、追踪进度、添加评分和评论。

## 概述

i-rs-read 提供以下核心功能：

- **书籍管理** - 添加、删除、查看书籍信息
- **进度追踪** - 记录当前页数，自动计算完成百分比
- **状态管理** - 支持多种阅读状态（阅读中、已完成、暂停、放弃、待读）
- **评分评论** - 为已读书籍打分和撰写评论
- **标签分类** - 使用标签组织和管理书籍
- **阅读统计** - 查看总书籍数、总页数、平均评分等统计信息

## 快速开始

### 安装

```bash
npm install -g @i-rs/i-rs-read
# 或者
brew install i-rs/homebrew-tap/i-rs-read
```

### 基本使用

```bash
# 添加新书籍
i-rs-read add "Rust 编程之道" "Steve Klabnik" 500

# 更新阅读进度
i-rs-read update "Rust 编程之道" --current-page 250

# 标记为已完成并评分
i-rs-read update "Rust 编程之道" --status completed --rating 5

# 列出所有书籍
i-rs-read list

# 查看阅读统计
i-rs-read stats
```

## 命令列表

| 命令 | 说明 |
|------|------|
| [add](./usage.html#add) | 添加新书籍 |
| [list](./usage.html#list) | 列出所有书籍 |
| [get](./usage.html#get) | 获取书籍详情 |
| [update](./usage.html#update) | 更新书籍信息 |
| [delete](./usage.html#delete) | 删除书籍 |
| [stats](./usage.html#stats) | 显示阅读统计 |
| [example](./usage.html#example) | 显示使用示例 |
| [skill](./usage.html#skill) | 显示 AI 技能文档 |

## 阅读状态

| 状态 | 说明 |
|------|------|
| `to_read` | 待读 |
| `reading` | 阅读中 |
| `completed` | 已完成 |
| `paused` | 暂停 |
| `dropped` | 放弃 |

## 数据存储

书籍数据存储在本地 JSON 文件中：

- **macOS**: `~/.config/i-rs/read.json`
- **Linux**: `~/.config/i-rs/read.json`
- **Windows**: `~\AppData\Roaming\i-rs\read.json`

可通过 `CONFIG_DIR` 环境变量覆盖配置目录。
