# i-rs-tax 使用文档

## Global Flags

- `--json` — Output in JSON format

## 命令概述

| 命令 | 说明 |
|------|------|
| `add` | 添加税务记录 |
| `list` | 列出税务记录 |
| `get` | 查看税务记录详情 |
| `delete` | 删除税务记录 |
| `stats` | 查看年度统计 |
| `example` | 显示使用示例 |
| `skill` | 显示 AI 技能文档 |

## add - 添加税务记录

```bash
i-rs-tax add <名称> [选项]
```

### 选项

| 选项 | 简写 | 说明 | 必需 |
|------|------|------|------|
| `--tax-type` | `-t` | 税种类型 (personal/vat) | 是 |
| `--amount` | `-a` | 金额 | 是 |
| `--date` | `-d` | 日期 (YYYY-MM-DD) | 是 |
| `--year` | `-y` | 年度 (默认从日期提取) | 否 |
| `--status` | `-s` | 报税状态 | 否 |
| `--tag` | | 标签 (可多次指定) | 否 |
| `--remark` | | 备注 (可多次指定) | 否 |
| `--json` | | JSON 格式输出 | 否 |

### 税种类型

- `personal` / `个人所得税`: 个人所得税
- `vat` / `增值税`: 增值税

### 报税状态

- `unreported` / `未申报` (默认)
- `filing` / `申报中`
- `filed` / `已申报`
- `paid` / `已缴纳`

### 示例

```bash
# 基本用法
i-rs-tax add 个税2024 --tax-type personal --amount 12000 --date 2024-03-15

# 完整选项
i-rs-tax add 个税2024 \
  --tax-type personal \
  --amount 12000 \
  --date 2024-03-15 \
  --year 2024 \
  --status filed \
  --tag 工资 \
  --tag 年终奖 \
  --remark 年终奖申报
```

## list - 列出税务记录

```bash
i-rs-tax list [选项]
```

### 选项

| 选项 | 简写 | 说明 |
|------|------|------|
| `--tag` | `-t` | 按标签过滤 |
| `--year` | `-y` | 按年度过滤 |
| `--tax-type` | | 按税种过滤 (personal/vat) |
| `--json` | | JSON 格式输出 |

### 示例

```bash
# 列出所有记录
i-rs-tax list

# 按年度筛选
i-rs-tax list --year 2024

# 按标签筛选
i-rs-tax list --tag 工资

# 按税种筛选
i-rs-tax list --tax-type personal

# 组合筛选
i-rs-tax list --year 2024 --tag 工资
```

## get - 查看详情

```bash
i-rs-tax get <名称> [选项]
```

### 选项

| 选项 | 说明 |
|------|------|
| `--json` | JSON 格式输出 |

### 示例

```bash
i-rs-tax get 个税2024
i-rs-tax get 个税2024 --json
```

## delete - 删除记录

```bash
i-rs-tax delete <名称> [选项]
```

### 选项

| 选项 | 说明 |
|------|------|
| `--json` | JSON 格式输出 |

### 示例

```bash
i-rs-tax delete 个税2024
```

## stats - 年度统计

```bash
i-rs-tax stats [选项]
```

### 选项

| 选项 | 简写 | 说明 |
|------|------|------|
| `--year` | `-y` | 指定年度 (默认当前年度) |
| `--tax-type` | `-t` | 按税种过滤 |
| `--json` | | JSON 格式输出 |

### 示例

```bash
# 当前年度统计
i-rs-tax stats

# 指定年度统计
i-rs-tax stats --year 2024

# 仅统计个人所得税
i-rs-tax stats --tax-type personal
```

## example - 使用示例

```bash
i-rs-tax example
```

显示完整的使用示例。

## skill - AI 技能文档

```bash
i-rs-tax skill [summary|content]
```

### 参数

- `summary`: 显示概要
- `content`: 显示完整内容 (默认)

### 示例

```bash
i-rs-tax skill
i-rs-tax skill summary
```

### data

Manage data (export, import, clear).

```bash
i-rs-tax data export
i-rs-tax data import [FILE]
i-rs-tax data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
### example

Show usage examples.

```bash
i-rs-tax example
```
### skill

Show skill information.

```bash
i-rs-tax skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/tax.json`
- Linux: `~/.config/i-rs/tax.json`
- Windows: `~\AppData\Roaming\i-rs\tax.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-tax list
```
