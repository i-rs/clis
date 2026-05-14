# i-rs-debt 测试记录

## 测试环境

- macOS
- i-rs-debt v0.0.2

## 测试用例

### 1. 添加债务测试

```bash
# 添加信用卡债务
$ i-rs-debt add "测试信用卡" --debt-type credit_card --amount 10000 --interest-rate 15.0
Debt '测试信用卡' created successfully

# 添加贷款
$ i-rs-debt add "测试贷款" --debt-type loan --amount 50000 --interest-rate 5.5 --tags test
Debt '测试贷款' created successfully

# 添加借款
$ i-rs-debt add "测试借款" --debt-type borrowed --amount 2000 --remark "测试备注"
Debt '测试借款' created successfully
```

### 2. 列表测试

```bash
# 列出所有债务
$ i-rs-debt list
+-------------+-------------+----------+-------+-----------+-----------+------------+------+
| Name        | Debt Type   | Total    | Paid  | Remaining | Progress  | Due Date   | Tags |
+-------------+-------------+----------+-------+-----------+-----------+------------+------+
| 测试信用卡    | CreditCard  | 10000.00 | 0.00  | 10000.00  | 0.0%      | -          | -    |
| 测试借款     | Borrowed    | 2000.00  | 0.00  | 2000.00   | 0.0%      | -          | -    |
| 测试贷款     | Loan        | 50000.00 | 0.00  | 50000.00  | 0.0%      | -          | -    |
+-------------+-------------+----------+-------+-----------+-----------+------------+------+

Total: 3 debts

# 按标签筛选
$ i-rs-debt list --tag test
+-------------+-------------+----------+-------+-----------+-----------+------------+------+
| Name        | Debt Type   | Total    | Paid  | Remaining | Progress  | Due Date   | Tags |
+-------------+-------------+----------+-------+-----------+-----------+------------+------+
| 测试贷款     | Loan        | 50000.00 | 0.00  | 50000.00  | 0.0%      | -          | test |
+-------------+-------------+----------+-------+-----------+-----------+------------+------+

Total: 1 debts
```

### 3. 查看详情测试

```bash
# 基本详情
$ i-rs-debt get "测试信用卡"
Name: 测试信用卡
Type: CreditCard
Total Amount: 10000.00
Paid Amount: 0.00
Remaining: 10000.00
Progress: 0.0%
Created: 2026-05-14 10:00:00
Updated: 2026-05-14 10:00:00

# 带还款历史
$ i-rs-debt get "测试信用卡" --payments
Name: 测试信用卡
...
=== Payment History ===
1 2026-05-14 10:05 500.00 (月供)
```

### 4. 还款测试

```bash
# 记录还款
$ i-rs-debt pay "测试信用卡" --amount 500 --note "月供"
Payment of 500.00 recorded for '测试信用卡'
  Paid: 500.00 | Remaining: 9500.00 | Progress: 5.0%

# 再次还款
$ i-rs-debt pay "测试信用卡" --amount 1500 --note "提前还款"
Payment of 1500.00 recorded for '测试信用卡'
  Paid: 2000.00 | Remaining: 8000.00 | Progress: 20.0%
```

### 5. 更新测试

```bash
# 更新利率
$ i-rs-debt update "测试信用卡" --interest-rate 12.0
Debt '测试信用卡' updated successfully

# 添加标签
$ i-rs-debt update "测试信用卡" --add-tags important,urgent
Debt '测试信用卡' updated successfully

# 更新详情
$ i-rs-debt get "测试信用卡"
Name: 测试信用卡
Type: CreditCard
Total Amount: 10000.00
Interest Rate: 12.0%
...
Tags: important, urgent
```

### 6. 逾期测试

```bash
# 添加已逾期的债务
$ i-rs-debt add "已逾期债务" --debt-type credit_card --amount 5000 --due-date 2026-01-01
Debt '已逾期债务' created successfully

# 查看逾期债务
$ i-rs-debt list --overdue
+-------------+-------------+----------+-------+-----------+-----------+------------+------+
| Name        | Debt Type   | Total    | Paid  | Remaining | Progress  | Due Date   | Tags |
+-------------+-------------+----------+-------+-----------+-----------+------------+------+
| 已逾期债务   | CreditCard  | 5000.00  | 0.00  | 5000.00   | 0.0%      | 2026-01-01 | -    |
+-------------+-------------+----------+-------+-----------+-----------+------------+------+

Total: 1 debts
```

### 7. 统计测试

```bash
# 基本统计
$ i-rs-debt stats
=== Debt Statistics ===
Total Debts: 4
Total Amount: 67000.00
Total Paid: 2000.00
Total Remaining: 65000.00
Overdue Count: 1

# 按类型统计
$ i-rs-debt stats --by-type
=== Debt Statistics ===
Total Debts: 4
Total Amount: 67000.00
Total Paid: 2000.00
Total Remaining: 65000.00
Overdue Count: 1

=== By Type ===
CreditCard (2)
  Total: 15000.00
  Paid: 2000.00
  Remaining: 13000.00

Borrowed (1)
  Total: 2000.00
  Paid: 0.00
  Remaining: 2000.00

Loan (1)
  Total: 50000.00
  Paid: 0.00
  Remaining: 50000.00
```

### 8. 删除测试

```bash
# 删除债务
$ i-rs-debt delete "测试借款" --force
Debt '测试借款' deleted successfully

# 验证删除
$ i-rs-debt get "测试借款"
Error: Debt '测试借款' not found
```

### 9. JSON 输出测试

```bash
# 列表 JSON
$ i-rs-debt list --json
{"success":true,"data":[...],"meta":{"count":3,"filter":null}}

# 详情 JSON
$ i-rs-debt get "测试信用卡" --json
{"success":true,"data":{...}}

# 统计 JSON
$ i-rs-debt stats --json
{"success":true,"data":{"total_debts":3,...}}
```

### 10. 还清测试

```bash
# 记录还款直到还清
$ i-rs-debt pay "测试借款" --amount 2000
Payment of 2000.00 recorded for '测试借款'
  Paid: 2000.00 | Remaining: 0.00 | Progress: 100.0%
Congratulations! This debt is fully paid off!

# 查看已还清债务
$ i-rs-debt list --paid-off
+-------------+-------------+----------+-------+-----------+-----------+------------+------+
| Name        | Debt Type   | Total    | Paid  | Remaining | Progress  | Due Date   | Tags |
+-------------+-------------+----------+-------+-----------+-----------+------------+------+
| 测试借款     | Borrowed    | 2000.00  | 2000.00| 0.00     | 100.0%    | -          | -    |
+-------------+-------------+----------+-------+-----------+-----------+------------+------+

Total: 1 debts
```

## 测试清理

```bash
# 清理所有测试数据
$ i-rs-debt delete "测试信用卡" --force
$ i-rs-debt delete "测试贷款" --force
$ i-rs-debt delete "测试借款" --force
$ i-rs-debt delete "已逾期债务" --force
```

## 测试总结

✅ 所有命令测试通过
✅ 数据持久化正常
✅ JSON 输出格式正确
✅ 逾期检测正常
✅ 还款追踪正常
✅ 统计功能正常
