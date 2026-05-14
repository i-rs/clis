# i-rs-budget 概述

个人预算管理 CLI 工具，帮助你跟踪预算和支出。

## 主要概念

### 预算 (Budget)

预算定义了一个类别的支出上限：

| 字段 | 说明 |
|------|------|
| category | 预算类别名称 |
| amount | 预算金额 |
| period | 周期 (daily/weekly/monthly/yearly) |
| tags | 标签列表 |
| remark | 备注 |

### 支出 (Expense)

支出是实际的消费记录：

| 字段 | 说明 |
|------|------|
| id | 唯一标识符 |
| category | 所属预算类别 |
| amount | 支出金额 |
| description | 支出描述 |
| date | 支出日期 |
| tags | 标签列表 |

## 快速开始

### 1. 添加预算

```bash
# 添加月度预算
i-rs-budget add food 500

# 添加周预算
i-rs-budget add groceries 300 --period weekly
```

### 2. 记录支出

```bash
# 记录一笔支出
i-rs-budget expense food 25.50 --description "午餐"
```

### 3. 查看统计

```bash
# 查看本月预算使用情况
i-rs-budget stats
```

## 命令列表

| 命令 | 说明 |
|------|------|
| `add` | 添加新预算 |
| `expense` | 记录支出 |
| `list` | 列出预算或支出 |
| `stats` | 查看预算统计 |
| `get` | 获取单个预算或支出详情 |
| `update` | 更新预算信息 |
| `delete` | 删除预算或支出 |
| `example` | 显示使用示例 |
| `skill` | 查看 AI 技能文档 |

## 数据存储

- **位置**: `~/.config/i-rs/budget.json`
- **格式**: JSON
- **覆盖**: 可通过 `CONFIG_DIR` 环境变量指定其他路径
