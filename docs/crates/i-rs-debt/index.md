# i-rs-debt 使用指南

## add - 添加债务

创建新的债务记录。

```bash
i-rs-debt add <name> --debt-type <type> --amount <amount> [options]
```

**必填参数:**
- `name`: 债务名称
- `--debt-type`: 债务类型 (credit_card, loan, borrowed)
- `--amount`: 债务总额

**可选参数:**
- `--interest-rate`: 年利率 (百分比)
- `--due-date`: 到期日期 (YYYY-MM-DD)
- `--tags`: 标签 (逗号分隔)
- `--remark`: 备注 (可多次使用)

**示例:**
```bash
i-rs-debt add "信用卡A" --debt-type credit_card --amount 10000 --interest-rate 15.0
i-rs-debt add "车贷" --debt-type loan --amount 50000 --tags car,vehicle --due-date 2026-12-31
```

---

## list - 列出债务

列出所有债务，支持筛选。

```bash
i-rs-debt list [options]
```

**可选参数:**
- `--tag`: 按标签筛选
- `--overdue`: 只显示逾期债务
- `--paid-off`: 只显示已还清债务

**示例:**
```bash
i-rs-debt list
i-rs-debt list --tag car
i-rs-debt list --overdue
```

---

## get - 查看债务详情

查看单个债务的详细信息。

```bash
i-rs-debt get <name> [options]
```

**可选参数:**
- `--payments`: 显示还款历史

**示例:**
```bash
i-rs-debt get "信用卡A"
i-rs-debt get "信用卡A" --payments
```

---

## pay - 记录还款

记录一笔还款。

```bash
i-rs-debt pay <name> --amount <amount> [options]
```

**必填参数:**
- `name`: 债务名称
- `--amount`: 还款金额

**可选参数:**
- `--note`: 还款备注

**示例:**
```bash
i-rs-debt pay "信用卡A" --amount 500 --note "月供"
```

---

## update - 更新债务

更新债务信息。

```bash
i-rs-debt update <name> [options]
```

**可选参数:**
- `--rename`: 新名称
- `--debt-type`: 新债务类型
- `--amount`: 新债务总额
- `--interest-rate`: 新利率
- `--due-date`: 新到期日期
- `--add-tags`: 添加标签
- `--remove-tags`: 移除标签
- `--add-remark`: 添加备注

**示例:**
```bash
i-rs-debt update "信用卡A" --interest-rate 12.0
i-rs-debt update "信用卡A" --add-tags important
```

---

## delete - 删除债务

删除债务记录。

```bash
i-rs-debt delete <name> [options]
```

**可选参数:**
- `--force`: 跳过确认直接删除

**示例:**
```bash
i-rs-debt delete "旧债务" --force
```

---

## stats - 统计数据

显示债务统计信息。

```bash
i-rs-debt stats [options]
```

**可选参数:**
- `--by-type`: 按债务类型分组显示

**示例:**
```bash
i-rs-debt stats
i-rs-debt stats --by-type
```

---

## 全局选项

- `--json`: 以 JSON 格式输出

---

## example - 使用示例

显示详细的使用示例。

```bash
i-rs-debt example
```

---

## skill - AI 技能文档

查看 AI 技能文档。

```bash
i-rs-debt skill      # 显示完整文档
i-rs-debt skill summary  # 显示摘要
```
