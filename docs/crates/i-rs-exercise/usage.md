# i-rs-exercise 使用指南

## Global Flags

- `--json` — Output in JSON format

## 命令概览

| 命令 | 描述 |
|------|------|
| `add` | 添加运动记录 |
| `list` | 列出运动记录 |
| `get` | 获取运动记录详情 |
| `update` | 更新运动记录 |
| `delete` | 删除运动记录 |
| `stats` | 显示统计数据 |
| `example` | 显示使用示例 |
| `skill` | 显示技能文档 |

---

## add

添加新的运动记录。

```bash
i-rs-exercise add <NAME> <TYPE> <DURATION> [OPTIONS]
```

### 参数

| 参数 | 描述 | 必填 |
|------|------|------|
| `NAME` | 运动名称 | 是 |
| `TYPE` | 运动类型（如 running、swimming、gym） | 是 |
| `DURATION` | 持续时间（分钟） | 是 |

### 选项

| 短选项 | 长选项 | 描述 |
|--------|--------|------|
| `-c` | `--calories` | 消耗卡路里 |
| `-t` | `--tag` | 标签（可多次使用） |
| `-n` | `--notes` | 备注（可多次使用） |
| `-r` | `--remark` | 备注（可多次使用） |

### 示例

```bash
# 添加跑步记录
i-rs-exercise add "Morning Run" running 30 -c 300

# 添加带标签的记录
i-rs-exercise add "Gym Session" gym 60 -c 500 -t strength -t upper-body

# 添加带备注的记录
i-rs-exercise add "Swimming" swimming 45 -c 400 -n "Lap swimming" -r "Feeling great!"
```

---

## list

列出运动记录，支持按标签或运动类型筛选。

```bash
i-rs-exercise list [OPTIONS]
```

### 选项

| 短选项 | 长选项 | 描述 |
|--------|--------|------|
| `-t` | `--tag` | 按标签筛选 |
| `-y` | `--exercise-type` | 按运动类型筛选 |

### 示例

```bash
# 列出所有记录
i-rs-exercise list

# 筛选有 cardio 标签的记录
i-rs-exercise list --tag cardio

# 筛选跑步类型记录
i-rs-exercise list --exercise-type running
```

### 表格输出示例

```
┌─────────────┬─────────┬──────────┬───────────┬─────────────┐
│ NAME        │ TYPE    │ DURATION │ CALORIES  │ TAGS        │
├─────────────┼─────────┼──────────┼───────────┼─────────────┤
│ Morning Run │ running │ 30 min   │ 300       │ morning     │
│ Gym Session │ gym     │ 60 min   │ 500       │ strength    │
└─────────────┴─────────┴──────────┴───────────┴─────────────┘

Total: 2 entities
```

---

## get

获取运动记录的详细信息。

```bash
i-rs-exercise get <NAME>
```

### 参数

| 参数 | 描述 | 必填 |
|------|------|------|
| `NAME` | 运动记录名称 | 是 |

### 示例

```bash
# 获取记录详情
i-rs-exercise get "Morning Run"
```

### 详细输出示例

```
Exercise: Morning Run
─────────────────────

Name:        Morning Run
Type:        running
Duration:    30 min
Calories:    300 kcal
Tags:        morning, cardio
Notes:       -
Remark:      Feeling great!
Created:     2025-01-15 08:00:00
Updated:     2025-01-15 08:00:00
```

---

## update

更新现有运动记录。

```bash
i-rs-exercise update <NAME> [OPTIONS]
```

### 参数

| 参数 | 描述 | 必填 |
|------|------|------|
| `NAME` | 要更新的记录名称 | 是 |

### 选项

| 短选项 | 长选项 | 描述 |
|--------|--------|------|
| `-y` | `--exercise-type` | 新运动类型 |
| `-d` | `--duration-minutes` | 新持续时间（分钟） |
| `-c` | `--calories` | 新卡路里值（使用空字符串清空） |
| `-t` | `--tag` | 新标签（覆盖现有标签） |
| `-n` | `--notes` | 新备注（覆盖现有备注） |
| `-r` | `--remark` | 新备注（覆盖现有备注） |

### 示例

```bash
# 更新持续时间
i-rs-exercise update "Morning Run" --duration-minutes 45

# 更新卡路里
i-rs-exercise update "Morning Run" --calories 350

# 清空卡路里
i-rs-exercise update "Morning Run" --calories ''

# 更新标签
i-rs-exercise update "Morning Run" -t cardio -t interval

# 同时更新多个字段
i-rs-exercise update "Morning Run" -d 50 -c 400 -t cardio
```

---

## delete

删除运动记录。

```bash
i-rs-exercise delete <NAME>
```

### 参数

| 参数 | 描述 | 必填 |
|------|------|------|
| `NAME` | 要删除的记录名称 | 是 |

### 示例

```bash
# 删除记录
i-rs-exercise delete "Morning Run"
```

---

## stats

显示运动统计数据，包括总时长、总卡路里、按类型统计等。

```bash
i-rs-exercise stats
```

### 统计输出示例

```
Exercise Statistics
────────────────────────────────────────

Total Records:         15
Total Duration:        720 min
Total Calories:        12500 kcal
Exercise Types:        5

Most Common:           running
Longest Type:          swimming

Breakdown by Type:

running              6 records      180 min  (avg 30.0 min)
swimming             3 records      150 min  (avg 50.0 min)
gym                  4 records      240 min  (avg 60.0 min)
cycling              2 records       90 min  (avg 45.0 min)
yoga                 1 records       60 min  (avg 60.0 min)
```

---

## example

显示使用示例，帮助快速了解 CLI 功能。

```bash
i-rs-exercise example
```

---

## skill

查看 AI 技能文档。

```bash
i-rs-exercise skill [SUB_COMMAND]
```

### 子命令

| 子命令 | 描述 |
|--------|------|
| 无参数 | 显示完整技能文档 |
| `summary` | 显示摘要 |
| `content` | 显示内容 |
| `raw` | 显示原始文档 |

---

## JSON 输出

所有命令支持 `--json` 全局标志：

```bash
i-rs-exercise list --json
i-rs-exercise get "Morning Run" --json
i-rs-exercise stats --json
```

### JSON 响应格式

**列表响应：**
```json
{
  "success": true,
  "data": [...],
  "meta": {
    "count": 10,
    "filter": "cardio"
  }
}
```

**项目响应：**
```json
{
  "success": true,
  "data": {...}
}
```

**错误响应：**
```json
{
  "success": false,
  "error": {
    "code": "NOT_FOUND",
    "message": "Entry 'xxx' not found"
  }
}
```

### data

Manage data (export, import, clear).

```bash
i-rs-exercise data export
i-rs-exercise data import [FILE]
i-rs-exercise data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
### example

Show usage examples.

```bash
i-rs-exercise example
```
### skill

Show skill information.

```bash
i-rs-exercise skill [summary|content|raw]
```
