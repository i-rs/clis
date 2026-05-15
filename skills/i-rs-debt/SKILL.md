---
name: "i-rs-debt"
description: "Debt management CLI tool. Track credit card debts, loans, borrowed money with payment history and overdue reminders."
---

# i-rs-debt

债务管理 CLI 工具，用于追踪信用卡债务、贷款和借款。

## Storage

- Config: `~/.config/i-rs/debt.json`

## Global Flags

- `--json` — Output in JSON format

## Commands

### add
创建新的债务记录。
```bash
i-rs-debt add <name> --debt-type <type> --amount <amount> [options]
```
Options:
- `--debt-type`: credit_card, loan, borrowed
- `--amount`: 债务总额
- `--interest-rate`: 利率百分比 (可选)
- `--due-date`: 到期日期 YYYY-MM-DD (可选)
- `--tags`: 标签，逗号分隔 (可选)
- `--remark`: 备注 (可选，可多次使用)

### list
列出所有债务。
```bash
i-rs-debt list [options]
```
Options:
- `--tag`: 按标签筛选
- `--overdue`: 只显示逾期债务
- `--paid-off`: 只显示已还清债务

### get
显示债务详细信息。
```bash
i-rs-debt get <name> [options]
```
Options:
- `--payments`: 显示还款历史

### pay
记录还款。
```bash
i-rs-debt pay <name> --amount <amount> [options]
```
Options:
- `--amount`: 还款金额
- `--note`: 还款备注 (可选)

### update
更新债务信息。
```bash
i-rs-debt update <name> [options]
```
Options:
- `--rename`: 新名称
- `--debt-type`: 新债务类型
- `--amount`: 新债务总额
- `--interest-rate`: 新利率
- `--due-date`: 新到期日期
- `--add-tags`: 添加标签 (逗号分隔)
- `--remove-tags`: 移除标签 (逗号分隔)
- `--add-remark`: 添加备注

### delete
删除债务记录。
```bash
i-rs-debt delete <name> [options]
```
Options:
- `--force`: 跳过确认

### stats
显示债务统计。
```bash
i-rs-debt stats [options]
```
Options:
- `--by-type`: 按债务类型分组显示

### example
显示使用示例。
```bash
i-rs-debt example
```

### skill
显示 AI 技能文档。
```bash
i-rs-debt skill
i-rs-debt skill summary
```

### data

Manage data (export, import, clear).

```bash
i-rs-debt data export
i-rs-debt data import [FILE]
i-rs-debt data clear
```

## Examples

```bash
# 添加信用卡债务
i-rs-debt add "信用卡A" --debt-type credit_card --amount 10000 --interest-rate 15.0 [OPTIONS]

# 添加贷款
i-rs-debt add "车贷" --debt-type loan --amount 50000 --tags car,vehicle [OPTIONS]

# 记录还款
i-rs-debt pay "信用卡A" --amount 500 --note "月供"

# 查看债务详情
i-rs-debt get "信用卡A" --payments

# 查看统计
i-rs-debt stats --by-type

# 列出逾期债务
i-rs-debt list --overdue
```
