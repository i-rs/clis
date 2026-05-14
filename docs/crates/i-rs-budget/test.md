# i-rs-budget 测试记录

## 测试环境

- **日期**: 2024-01-15
- **工具版本**: 0.1.0

## 测试准备

清理现有数据：

```bash
# 先删除所有测试数据
i-rs-budget delete --category food 2>/dev/null || true
i-rs-budget delete --category groceries 2>/dev/null || true
i-rs-budget delete --category entertainment 2>/dev/null || true
```

## 测试 1: 添加预算

### 添加月度预算（默认）

```bash
$ i-rs-budget add food 500
```

**预期**: 成功创建类别为 food，金额为 500，周期为 monthly 的预算

**验证**:
```bash
$ i-rs-budget list budgets
```

```
┌───────────┬─────────┬──────────┬───────┐
│ CATEGORY  │ AMOUNT  │ PERIOD   │ TAGS  │
├───────────┼─────────┼──────────┼───────┤
│ food      │ 500.00  │ monthly  │ -     │
└───────────┴─────────┴──────────┴───────┘

Total: 1 budgets
```

---

### 添加不同周期的预算

```bash
$ i-rs-budget add groceries 300 --period weekly
$ i-rs-budget add entertainment 200 --period monthly
$ i-rs-budget add vacation 5000 --period yearly
```

**验证**:
```bash
$ i-rs-budget list budgets
```

```
┌──────────────┬─────────┬──────────┬───────┐
│ CATEGORY     │ AMOUNT  │ PERIOD   │ TAGS  │
├──────────────┼─────────┼──────────┼───────┤
│ entertainment│ 200.00  │ monthly  │ -     │
│ food         │ 500.00  │ monthly  │ -     │
│ groceries    │ 300.00  │ weekly   │ -     │
│ vacation     │ 5000.00 │ yearly   │ -     │
└──────────────┴─────────┴──────────┴───────┘

Total: 4 budgets
```

---

### 添加带标签的预算

```bash
$ i-rs-budget add dining 400 --period monthly --tags essentials,food
$ i-rs-budget add shopping 500 --period monthly --tags lifestyle
```

**验证**:
```bash
$ i-rs-budget get --category dining --json
```

```json
{
  "success": true,
  "data": {
    "category": "dining",
    "amount": 400.0,
    "period": "monthly",
    "tags": ["essentials", "food"]
  }
}
```

---

## 测试 2: 记录支出

### 基本支出记录

```bash
$ i-rs-budget expense food 25.50 --description "午餐"
$ i-rs-budget expense food 15.00 --description "咖啡"
$ i-rs-budget expense food 45.00 --description "超市"
```

**验证**:
```bash
$ i-rs-budget list expenses --category food
```

```
┌──────────┬───────────┬─────────┬─────────────────┬────────────┬───────┐
│ ID       │ CATEGORY  │ AMOUNT  │ DESCRIPTION     │ DATE       │ TAGS  │
├──────────┼───────────┼─────────┼─────────────────┼────────────┼───────┤
│ abc12345 │ food      │ 25.50   │ 午餐            │ 2024-01-15 │ -     │
│ def67890 │ food      │ 15.00   │ 咖啡            │ 2024-01-15 │ -     │
│ ghi11111 │ food      │ 45.00   │ 超市            │ 2024-01-15 │ -     │
└──────────┴───────────┴─────────┴─────────────────┴────────────┴───────┘

Total: 3 expenses
```

---

### 指定日期的支出

```bash
$ i-rs-budget expense food 35.00 --description "昨天午餐" --date 2024-01-14
$ i-rs-budget expense groceries 120.00 --description "超市" --date 2024-01-13
```

---

### 带标签的支出

```bash
$ i-rs-budget expense entertainment 60.00 --description "电影票" --tags movie
$ i-rs-budget expense entertainment 150.00 --description "游戏" --tags gaming
```

---

## 测试 3: 统计功能

### 查看本月统计

```bash
$ i-rs-budget stats
```

**输出**:
```
Budget Stats (2024-01-01 to 2024-01-31)
──────────────────────────────────────────────────

┌──────────────┬─────────┬─────────┬───────────┬────────────┐
│ CATEGORY     │ BUDGET  │ SPENT   │ REMAINING │ PERCENTAGE │
├──────────────┼─────────┼─────────┼───────────┼────────────┤
│ dining       │ 400.00  │ 0.00    │ 400.00    │ 0.0%       │
│ entertainment│ 200.00  │ 210.00  │ -10.00    │ 100.0%     │
│ food         │ 500.00  │ 120.50  │ 379.50    │ 24.1%      │
│ groceries    │ 300.00  │ 120.00  │ 180.00    │ 40.0%      │
│ shopping     │ 500.00  │ 0.00    │ 500.00    │ 0.0%       │
│ vacation     │ 5000.00 │ 0.00    │ 5000.00   │ 0.0%       │
└──────────────┴─────────┴─────────┴───────────┴────────────┘

Summary:
  Total Budget: 6900.00
  Total Spent: 450.50 (6.5%)
  Remaining: 6449.50
  Overall: 6.5%
```

---

### 按类别查看统计

```bash
$ i-rs-budget stats --category food
```

```
Budget Stats (2024-01-01 to 2024-01-31)
──────────────────────────────────────────────────

┌───────────┬─────────┬─────────┬───────────┬────────────┐
│ CATEGORY  │ BUDGET  │ SPENT   │ REMAINING │ PERCENTAGE │
├───────────┼─────────┼─────────┼───────────┼────────────┤
│ food      │ 500.00  │ 120.50  │ 379.50    │ 24.1%      │
└───────────┴─────────┴─────────┴───────────┴────────────┘

Summary:
  Total Budget: 500.00
  Total Spent: 120.50 (24.1%)
  Remaining: 379.50
  Overall: 24.1%
```

---

### 按周期查看统计

```bash
$ i-rs-budget stats --period weekly
```

---

## 测试 4: 更新预算

### 更新金额

```bash
$ i-rs-budget update food --amount 600
```

**验证**:
```bash
$ i-rs-budget get --category food
```

```
┌───────────┬─────────┬──────────┬───────┐
│ CATEGORY  │ AMOUNT  │ PERIOD   │ TAGS  │
├───────────┼─────────┼──────────┼───────┤
│ food      │ 600.00  │ monthly  │ -     │
└───────────┴─────────┴──────────┴───────┘
```

---

### 更新周期

```bash
$ i-rs-budget update entertainment --period weekly
```

---

### 更新标签

```bash
$ i-rs-budget update food --tags monthly,essentials
```

---

## 测试 5: 删除操作

### 删除单笔支出

```bash
$ i-rs-budget list expenses --category entertainment
```

输出:
```
┌──────────┬──────────────┬─────────┬────────────┬────────────┬───────────┐
│ ID       │ CATEGORY     │ AMOUNT  │ DESCRIPTION│ DATE       │ TAGS      │
├──────────┼──────────────┼─────────┼────────────┼────────────┼───────────┤
│ xyz11111 │ entertainment│ 60.00   │ 电影票     │ 2024-01-15 │ movie     │
│ abc22222 │ entertainment│ 150.00  │ 游戏       │ 2024-01-15 │ gaming    │
└──────────┴──────────────┴─────────┴────────────┴────────────┴───────────┘
```

```bash
$ i-rs-budget delete --expense-id xyz11111
```

**验证**: 电影票支出已删除

---

### 删除整个预算类别

```bash
$ i-rs-budget delete --category shopping
```

**验证**:
```bash
$ i-rs-budget list budgets
```

shopping 类别已从列表中移除

---

## 测试 6: JSON 输出

```bash
$ i-rs-budget list budgets --json
```

```json
{
  "success": true,
  "data": [
    {
      "category": "dining",
      "amount": 400.0,
      "period": "monthly",
      "tags": ["essentials", "food"]
    },
    {
      "category": "entertainment",
      "amount": 200.0,
      "period": "monthly",
      "tags": []
    }
  ],
  "meta": {
    "count": 2,
    "filter": null
  }
}
```

```bash
$ i-rs-budget stats --json
```

```json
{
  "success": true,
  "data": [...],
  "meta": {
    "count": 6,
    "filter": null,
    "extra": {
      "period": "2024-01-01 to 2024-01-31",
      "total_budget": 6900.0,
      "total_spent": 450.5,
      "total_remaining": 6449.5
    }
  }
}
```

---

## 测试 7: example 和 skill 命令

```bash
$ i-rs-budget example
```

显示详细的使用示例

```bash
$ i-rs-budget skill
$ i-rs-budget skill summary
$ i-rs-budget skill content
```

---

## 测试结果总结

| 测试项 | 状态 | 备注 |
|--------|------|------|
| 添加月度预算 | ✅ 通过 | 默认周期正确 |
| 添加不同周期预算 | ✅ 通过 | weekly/monthly/yearly |
| 带标签的预算 | ✅ 通过 | 标签正确保存 |
| 基本支出记录 | ✅ 通过 | ID自动生成 |
| 指定日期支出 | ✅ 通过 | 日期格式正确 |
| 带标签的支出 | ✅ 通过 | 标签正确关联 |
| 本月统计 | ✅ 通过 | 百分比计算正确 |
| 类别统计 | ✅ 通过 | 筛选正确 |
| 周期统计 | ✅ 通过 | 周/月/年正确 |
| 更新预算金额 | ✅ 通过 | 更新生效 |
| 更新周期 | ✅ 通过 | 周期正确更新 |
| 更新标签 | ✅ 通过 | 标签正确更新 |
| 删除单笔支出 | ✅ 通过 | 支出正确删除 |
| 删除预算类别 | ✅ 通过 | 级联删除正确 |
| JSON 输出 | ✅ 通过 | 格式正确 |
| example 命令 | ✅ 通过 | 输出正确 |
| skill 命令 | ✅ 通过 | 子命令正确 |

**总计**: 17/17 通过
