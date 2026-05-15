---
name: "i-rs-budget"
description: "个人预算管理工具。用于设置预算、记录支出、查看统计。适用于需要追踪日常开支、管理分类预算的场景。"
---

# i-rs-budget

个人预算管理 CLI 工具，帮助用户设置预算类别、记录支出、查看统计分析。

## 存储

- **配置文件**: `~/.config/i-rs/budget.json`
- **环境变量**: `CONFIG_DIR` 可覆盖默认路径
- **数据格式**: JSON，包含 budgets 和 expenses 两个集合

## Global Flags

- `--json` — Output in JSON format

## 核心概念

### Budget (预算)

定义一个类别的支出上限:

- `category`: 预算类别名称
- `amount`: 预算金额
- `period`: 周期 (daily/weekly/monthly/yearly)
- `tags`: 标签列表
- `remark`: 备注

### Expense (支出)

记录实际消费:

- `id`: 唯一标识符 (UUID 前8位)
- `category`: 所属预算类别
- `amount`: 支出金额
- `description`: 支出描述
- `date`: 支出日期
- `tags`: 标签列表

## 命令

### add

添加新预算:

```bash
i-rs-budget add <CATEGORY> <AMOUNT> [OPTIONS]
```

选项:
- `-p, --period <PERIOD>` - 预算周期: daily, weekly, monthly, yearly
- `-t, --tags <TAGS>` - 标签列表
- `-r, --remark <REMARK>` - 备注信息

示例:
```bash
i-rs-budget add food 500                              # 月度预算
i-rs-budget add groceries 300 --period weekly         # 周预算
i-rs-budget add vacation 5000 --period yearly          # 年预算
i-rs-budget add entertainment 200 --tags fun,leisure    # 带标签
```

### expense

记录支出:

```bash
i-rs-budget expense <CATEGORY> <AMOUNT> [OPTIONS]
```

选项:
- `-d, --description <DESC>` - 支出描述（必填）
- `--date <DATE>` - 支出日期，格式: YYYY-MM-DD
- `-t, --tags <TAGS>` - 标签列表

示例:
```bash
i-rs-budget expense food 25.50 --description "午餐"
i-rs-budget expense groceries 120.00 --date 2024-01-15
i-rs-budget expense entertainment 60.00 --tags movie
```

### list

列出预算或支出:

```bash
i-rs-budget list [TYPE] [OPTIONS]
```

选项:
- `-c, --category <CAT>` - 按类别筛选

示例:
```bash
i-rs-budget list budgets                              # 列出所有预算
i-rs-budget list expenses                             # 列出所有支出
i-rs-budget list expenses --category food             # 按类别筛选
```

### stats

查看预算统计:

```bash
i-rs-budget stats [OPTIONS]
```

选项:
- `-c, --category <CAT>` - 按类别筛选
- `-p, --period <PERIOD>` - 统计周期: daily, weekly, monthly, yearly

示例:
```bash
i-rs-budget stats                                     # 本月统计
i-rs-budget stats --period weekly                     # 本周统计
i-rs-budget stats --category food                    # 特定类别统计
i-rs-budget stats --category food --period monthly    # 组合筛选
```

### get

获取单个预算或支出详情:

```bash
i-rs-budget get [OPTIONS]
```

选项:
- `-c, --category <CAT>` - 获取指定预算类别
- `--expense-id <ID>` - 获取指定支出记录

### update

更新预算信息:

```bash
i-rs-budget update <CATEGORY> [OPTIONS]
```

选项:
- `-a, --amount <AMOUNT>` - 新的预算金额
- `-p, --period <PERIOD>` - 新的预算周期
- `-t, --tags <TAGS>` - 新的标签列表
- `-r, --remark <REMARK>` - 新的备注信息

示例:
```bash
i-rs-budget update food --amount 600                  # 更新金额
i-rs-budget update food --period weekly              # 更新周期
i-rs-budget update food --tags essentials             # 更新标签
```

### delete

删除预算或支出:

```bash
i-rs-budget delete [OPTIONS]
```

选项:
- `-c, --category <CAT>` - 删除指定预算类别（包含所有相关支出）
- `--expense-id <ID>` - 删除单笔支出

### data

Manage data (export, import, clear).

```bash
i-rs-budget data export
i-rs-budget data import [FILE]
i-rs-budget data clear
```

### example

显示使用示例:

```bash
i-rs-budget example
```

### skill

查看 AI 技能文档:

```bash
i-rs-budget skill [summary|content|raw]
```

## 常见使用场景

### 月度预算追踪

```bash
# 设置月度预算
i-rs-budget add food 1000 --period monthly [OPTIONS]
i-rs-budget add groceries 500 --period monthly [OPTIONS]
i-rs-budget add entertainment 300 --period monthly [OPTIONS]

# 日常记录
i-rs-budget expense food 35.00 --description "午餐"
i-rs-budget expense groceries 150.00 --description "超市"

# 查看进度
i-rs-budget stats --period monthly
```

### 周预算控制

```bash
# 设置周预算
i-rs-budget add groceries 300 --period weekly [OPTIONS]

# 周内记录
i-rs-budget expense groceries 80.00 --description "周一采购"
i-rs-budget expense groceries 120.00 --description "周三补货"

# 查看剩余
i-rs-budget stats --period weekly --category groceries
```

### 消费分析

```bash
# 查看所有支出
i-rs-budget list expenses

# 按类别分析
i-rs-budget stats --category food
i-rs-budget stats --category entertainment

# 年度总览
i-rs-budget stats --period yearly
```
