# i-rs-read

阅读进度追踪 CLI 工具，帮助你追踪和管理正在阅读的书籍。

## 功能特性

- 添加和管理书籍信息（书名、作者、总页数）
- 追踪阅读进度（当前页数、完成百分比）
- 支持多种阅读状态（Reading、Completed、Paused、Dropped、ToRead）
- 书籍评分和评论功能
- 标签分类支持
- 阅读统计（总书籍数、总页数、已完成数量、平均评分等）
- 支持 JSON 输出格式

## 安装

```bash
npm install -g @i-rs/i-rs-read
# 或者
brew install i-rs/homebrew-tap/i-rs-read
```

## 快速开始

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

## 命令说明

| 命令 | 说明 |
|------|------|
| `add` | 添加新书籍 |
| `list` | 列出所有书籍 |
| `get` | 获取书籍详情 |
| `update` | 更新书籍信息 |
| `delete` | 删除书籍 |
| `stats` | 显示阅读统计 |
| `example` | 显示使用示例 |
| `skill` | 显示 AI 技能文档 |

### add - 添加书籍

```bash
i-rs-read add <书名> <作者> <总页数> [选项]

选项:
  --tags <标签>       添加标签（可多次指定）
  --remark <备注>     添加备注（可多次指定）
```

### list - 列出书籍

```bash
i-rs-read list [选项]

选项:
  --tag <标签>        按标签筛选
  --status <状态>     按状态筛选（reading, completed, paused, dropped, to_read）
```

### update - 更新书籍

```bash
i-rs-read update <书名> [选项]

选项:
  --current-page <页码>  更新当前页数
  --status <状态>        更新阅读状态
  --rating <评分>        添加评分（0-5）
  --review <评论>         添加评论
  --tags <标签>          更新标签（逗号分隔）
  --add-remark <备注>    添加备注
  --remove-remark <索引>  删除备注（1-based）
```

### stats - 阅读统计

```bash
i-rs-read stats [选项]

选项:
  --tag <标签>  按标签筛选统计
```

## 数据存储

- macOS: `~/.config/i-rs/read.json`
- Linux: `~/.config/i-rs/read.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

可通过 `CONFIG_DIR` 环境变量覆盖配置目录。

## JSON 输出

所有命令支持 `--json` 全局标志以 JSON 格式输出：

```bash
i-rs-read list --json
i-rs-read get "书名" --json
i-rs-read stats --json
```

## License

MIT OR Apache-2.0
