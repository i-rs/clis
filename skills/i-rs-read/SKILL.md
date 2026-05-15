---
name: "i-rs-read"
description: "阅读进度追踪 CLI 工具，用于管理书籍、追踪阅读进度、添加评分和评论。适用于需要追踪阅读计划、管理待读书单、记录阅读进度的场景。"
---

# i-rs-read

## Global Flags

- `--json` — Output in JSON format


i-rs-read 是一个阅读进度追踪 CLI 工具，帮助用户管理书籍、追踪阅读进度、添加评分和评论。

## 存储

- **配置文件**: `~/.config/i-rs/read.json`
- **环境变量**: 可通过 `CONFIG_DIR` 覆盖配置目录

## 命令

### add

添加一本新书。

```bash
i-rs-read add <书名> <作者> <总页数> [选项]

选项:
  --tags <标签>       添加标签
  --remark <备注>     添加备注
```

### list

列出所有书籍。

```bash
i-rs-read list [选项]

选项:
  --tag <标签>        按标签筛选
  --status <状态>     按状态筛选
```

### get

获取书籍详情。

```bash
i-rs-read get <书名>
```

### update

更新书籍信息。

```bash
i-rs-read update <书名> [选项]

选项:
  --current-page <页码>  更新当前页数
  --status <状态>        更新状态 (reading, completed, paused, dropped, to_read)
  --rating <评分>        添加评分 (0-5)
  --review <评论>        添加评论
  --tags <标签>          更新标签
  --add-remark <备注>    添加备注
  --remove-remark <索引> 删除备注
```

### delete

删除书籍。

```bash
i-rs-read delete <书名>
```

### stats

显示阅读统计。

```bash
i-rs-read stats [选项]

选项:
  --tag <标签>  按标签筛选统计
```

### example

显示使用示例。

```bash
i-rs-read example
```

### skill

显示 AI 技能文档。

```bash
i-rs-read skill [--summary] [--content]
```

## 阅读状态

| 状态 | 说明 |
|------|------|
| `to_read` | 待读 |
| `reading` | 阅读中 |
| `completed` | 已完成 |
| `paused` | 暂停 |
| `dropped` | 放弃 |

## 数据结构

```json
{
  "books": {
    "书名": {
      "name": "书名",
      "author": "作者",
      "total_pages": 500,
      "current_page": 250,
      "status": "reading",
      "rating": 4.5,
      "review": "评论内容",
      "tags": ["tag1", "tag2"],
      "remark": ["备注1", "备注2"],
      "created_at": 1234567890,
      "updated_at": 1234567890
    }
  }
}
```

## 示例

```bash
# 添加书籍

## Global Flags

- `--json` — Output in JSON format

i-rs-read add "Rust 编程之道" "Steve Klabnik" 500

# 更新阅读进度

## Global Flags

- `--json` — Output in JSON format

i-rs-read update "Rust 编程之道" --current-page 250

# 标记为已完成并评分

## Global Flags

- `--json` — Output in JSON format

i-rs-read update "Rust 编程之道" --status completed --rating 5

# 列出所有书籍

## Global Flags

- `--json` — Output in JSON format

i-rs-read list

# 按标签筛选

## Global Flags

- `--json` — Output in JSON format

i-rs-read list --tag programming

# 查看统计

## Global Flags

- `--json` — Output in JSON format

i-rs-read stats
```

### data

Manage data (export, import, clear).

```bash
i-rs-read data export
i-rs-read data import [FILE]
i-rs-read data clear
```

## Examples

```bash
# JSON output

## Global Flags

- `--json` — Output in JSON format

i-rs-read list --json
```
