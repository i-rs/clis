# i-rs-budget 使用指南

## Global Flags

- `--json` — Output in JSON format

## add - 添加预算

创建新的预算类别。

```bash
i-rs-budget add <CATEGORY> <AMOUNT> [OPTIONS]
```

**参数:**

- `CATEGORY` - 预算类别名称
- `AMOUNT` - 预算金额（数字）

**选项:**

| 选项 | 说明 |
|------|------|
| `-p, --period <PERIOD>` | 预算周期: daily, weekly, monthly, yearly |
| `-t, --tags <TAGS>...` | 标签列表 |
| `-r, --remark <REMARK>...` | 备注信息 |

**示例:**

```bash
# 添加月度预算
i-rs-budget add food 500

# 添加周预算
i-rs-budget add groceries 300 --period weekly

# 添加带标签的预算
i-rs-budget add entertainment 200 --tags fun,leisure
```

---

## expense - 记录支出

记录一笔支出。

```bash
i-rs-budget expense <CATEGORY> <AMOUNT> [OPTIONS]
```

**参数:**

- `CATEGORY` - 支出所属的预算类别
- `AMOUNT` - 支出金额（数字）

**选项:**

| 选项 | 说明 |
|------|------|
| `-d, --description <DESC>` | 支出描述（必填） |
| `--date <DATE>` | 支出日期，格式: YYYY-MM-DD |
| `-t, --tags <TAGS>...` | 标签列表 |

**示例:**

```bash
# 记录支出
i-rs-budget expense food 25.50 --description "午餐"

# 指定日期
i-rs-budget expense groceries 120.30 --date 2024-01-15

# 带标签
i-rs-budget expense entertainment 50 --description "电影票" --tags movie
```

---

## list - 列出预算或支出

列出所有预算或支出记录。

```bash
i-rs-budget list [TYPE] [OPTIONS]
```

**参数:**

- `TYPE` - 列表类型: budgets, expenses（默认: budgets）

**选项:**

| 选项 | 说明 |
|------|------|
| `-c, --category <CAT>` | 按类别筛选 |

**示例:**

```bash
# 列出所有预算
i-rs-budget list budgets

# 列出所有支出
i-rs-budget list expenses

# 按类别列出支出
i-rs-budget list expenses --category food
```

---

## stats - 查看统计

查看预算使用情况统计。

```bash
i-rs-budget stats [OPTIONS]
```

**选项:**

| 选项 | 说明 |
|------|------|
| `-c, --category <CAT>` | 按类别筛选 |
| `-p, --period <PERIOD>` | 统计周期: daily, weekly, monthly, yearly |

**示例:**

```bash
# 查看本月统计
i-rs-budget stats

# 按周查看
i-rs-budget stats --period weekly

# 按类别查看
i-rs-budget stats --category food

# 组合筛选
i-rs-budget stats --category food --period monthly
```

---

## get - 获取详情

获取单个预算或支出的详细信息。

```bash
i-rs-budget get [OPTIONS]
```

**选项:**

| 选项 | 说明 |
|------|------|
| `-c, --category <CAT>` | 获取指定预算类别 |
| `--expense-id <ID>` | 获取指定支出记录 |

**示例:**

```bash
# 获取预算详情
i-rs-budget get --category food

# 获取支出详情
i-rs-budget get --expense-id abc12345
```

---

## update - 更新预算

更新现有预算的信息。

```bash
i-rs-budget update <CATEGORY> [OPTIONS]
```

**参数:**

- `CATEGORY` - 要更新的预算类别

**选项:**

| 选项 | 说明 |
|------|------|
| `-a, --amount <AMOUNT>` | 新的预算金额 |
| `-p, --period <PERIOD>` | 新的预算周期 |
| `-t, --tags <TAGS>...` | 新的标签列表 |
| `-r, --remark <REMARK>...` | 新的备注信息 |

**示例:**

```bash
# 更新金额
i-rs-budget update food --amount 600

# 更新周期
i-rs-budget update food --period weekly

# 更新标签
i-rs-budget update food --tags weekly,essentials
```

---

## delete - 删除

删除预算或支出记录。

```bash
i-rs-budget delete [OPTIONS]
```

**选项:**

| 选项 | 说明 |
|------|------|
| `-c, --category <CAT>` | 删除指定预算类别（及其所有支出） |
| `--expense-id <ID>` | 删除指定支出记录 |

**示例:**

```bash
# 删除预算
i-rs-budget delete --category food

# 删除单笔支出
i-rs-budget delete --expense-id abc12345
```

---

### data

Manage data (export, import, clear).

```bash
i-rs-budget data export
i-rs-budget data import [FILE]
i-rs-budget data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data

### example

显示详细的使用示例。

```bash
i-rs-budget example
```

### skill

查看 AI 技能文档。

```bash
i-rs-budget skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/budget.json`
- Linux: `~/.config/i-rs/budget.json`
- Windows: `~\AppData\Roaming\i-rs\budget.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-budget list
```
