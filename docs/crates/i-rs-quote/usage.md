# i-rs-quote 使用指南

## Global Flags

- `--json` — Output in JSON format

## 命令概述

i-rs-quote 提供以下命令来管理你的语录收藏。

## add - 添加语录

添加新的语录到收藏夹。

```bash
i-rs-quote add --content "语录内容" [OPTIONS]
```

### 选项

| 选项 | 简写 | 描述 | 必需 |
|------|------|------|------|
| `--content` | `-c` | 语录内容 | 是 |
| `--author` | `-a` | 作者姓名 | 否 |
| `--source` | `-s` | 出处（书籍、演讲等） | 否 |
| `--tag` | `-t` | 标签（可多次使用） | 否 |
| `--remark` | `-r` | 个人备注（可多次使用） | 否 |

### 示例

```bash
# 基本添加
i-rs-quote add --content "Stay hungry, stay foolish."

# 完整信息
i-rs-quote add \
  --content "The only way to do great work is to love what you do." \
  --author "Steve Jobs" \
  --source "Stanford Commencement Speech" \
  --tag inspiration \
  --tag life

# 带备注
i-rs-quote add \
  --content "Be the change you wish to see in the world." \
  --author "Mahatma Gandhi" \
  --tag wisdom \
  --remark "来自课堂引用"
```

## list - 列出语录

列出所有语录或按条件筛选。

```bash
i-rs-quote list [OPTIONS]
```

### 选项

| 选项 | 简写 | 描述 |
|------|------|------|
| `--tag` | `-t` | 按标签筛选 |
| `--author` | `-a` | 按作者筛选（模糊匹配） |

### 示例

```bash
# 列出所有
i-rs-quote list

# 按标签筛选
i-rs-quote list --tag inspiration

# 按作者搜索
i-rs-quote list --author "Steve"

# 组合筛选
i-rs-quote list --tag wisdom --author "Gandhi"
```

## get - 查看详情

查看单条语录的完整信息。

```bash
i-rs-quote get <ID>
```

### 示例

```bash
i-rs-quote get 550e8400-e29b-41d4-a716-446655440000
```

## delete - 删除语录

从收藏中删除一条语录。

```bash
i-rs-quote delete <ID>
```

### 示例

```bash
i-rs-quote delete 550e8400-e29b-41d4-a716-446655440000
```

## random - 随机展示

随机展示一条收藏的语录。

```bash
i-rs-quote random
```

### 示例

```bash
i-rs-quote random
```

## JSON 输出

所有命令支持 `--json` 全局标志，以 JSON 格式输出结果。

```bash
i-rs-quote list --json
i-rs-quote get <ID> --json
```

### JSON 响应格式

**列表响应:**
```json
{
  "success": true,
  "data": [...],
  "meta": {
    "count": 10,
    "filter": "tag:inspiration"
  }
}
```

**单条响应:**
```json
{
  "success": true,
  "data": {
    "id": "...",
    "content": "...",
    "author": "...",
    "source": "...",
    "tags": [...],
    "remark": [...],
    "created_at": "..."
  }
}
```

## 全局选项

| 选项 | 简写 | 描述 |
|------|------|------|
| `--json` | `-j` | 以 JSON 格式输出 |

## 环境变量

- `CONFIG_DIR`: 自定义配置目录路径

### data

Manage data (export, import, clear).

```bash
i-rs-quote data export
i-rs-quote data import [FILE]
i-rs-quote data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
### example

Show usage examples.

```bash
i-rs-quote example
```
### skill

Show skill information.

```bash
i-rs-quote skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/quote.json`
- Linux: `~/.config/i-rs/quote.json`
- Windows: `~\AppData\Roaming\i-rs\quote.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-quote list
```
