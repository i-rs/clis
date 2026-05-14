# i-rs-debt 使用示例

## 信用卡管理

### 追踪多张信用卡

```bash
# 添加招商银行信用卡
i-rs-debt add "招商银行信用卡" \
  --debt-type credit_card \
  --amount 20000 \
  --interest-rate 18.0 \
  --tags bank,credit-card \
  --remark "账单日: 每月5日" \
  --remark "还款日: 每月23日"

# 添加工商银行信用卡
i-rs-debt add "工商银行信用卡" \
  --debt-type credit_card \
  --amount 15000 \
  --interest-rate 15.0 \
  --tags bank,credit-card

# 列出所有信用卡
i-rs-debt list --tag credit-card

# 查看工商银行卡详情
i-rs-debt get "工商银行信用卡" --payments
```

### 记录每月还款

```bash
# 月初还最低还款额
i-rs-debt pay "招商银行信用卡" --amount 500 --note "最低还款"

# 资金充裕时多还
i-rs-debt pay "招商银行信用卡" --amount 2000 --note "全额还款"

# 查看还款历史
i-rs-debt get "招商银行信用卡" --payments
```

---

## 贷款管理

### 汽车贷款

```bash
# 添加车贷
i-rs-debt add "车贷" \
  --debt-type loan \
  --amount 150000 \
  --interest-rate 4.5 \
  --due-date 2029-05-01 \
  --tags car,vehicle,loan \
  --remark "贷款银行: XX银行" \
  --remark "月供: 3500元"

# 每月记录还款
i-rs-debt pay "车贷" --amount 3500 --note "2026年5月月供"

# 查看还款进度
i-rs-debt get "车贷"

# 查看贷款统计
i-rs-debt stats --by-type
```

### 房贷

```bash
# 添加房贷
i-rs-debt add "房贷" \
  --debt-type loan \
  --amount 3000000 \
  --interest-rate 3.9 \
  --due-date 2056-05-01 \
  --tags house,property,long-term \
  --remark "贷款银行: XX银行" \
  --remark "月供: 15000元"

# 记录还款
i-rs-debt pay "房贷" --amount 15000 --note "2026年5月月供"
```

---

## 借款管理

### 朋友借款

```bash
# 记录朋友借款
i-rs-debt add "张三借款" \
  --debt-type borrowed \
  --amount 5000 \
  --due-date 2026-12-31 \
  --tags personal,friends \
  --remark "向好友张三借款" \
  --remark "约定年底还清"

# 记录部分还款
i-rs-debt pay "张三借款" --amount 2000 --note "还一半"

# 记录全额还款
i-rs-debt pay "张三借款" --amount 3000 --note "还清尾款"

# 查看是否还清
i-rs-debt list --paid-off
```

### 家人借款

```bash
# 记录家人借款
i-rs-debt add "家人借款" \
  --debt-type borrowed \
  --amount 30000 \
  --tags family,no-interest \
  --remark "家人的钱，不急慢慢还"
```

---

## 逾期管理

### 查看逾期债务

```bash
# 查看所有逾期债务
i-rs-debt list --overdue

# 更新逾期债务的到期日
i-rs-debt update "张三借款" --due-date 2026-08-01

# 设置提醒（结合 i-rs-remind）
# i-rs-remind add "还张三借款" --date 2026-08-01 --repeat monthly
```

### 避免逾期

```bash
# 设置合理的到期日
i-rs-debt update "信用卡A" --due-date 2026-06-23

# 定期检查债务状态
i-rs-debt stats
```

---

## 标签管理

### 使用标签分类

```bash
# 按类型分类
i-rs-debt list --tag credit-card
i-rs-debt list --tag loan
i-rs-debt list --tag borrowed

# 按用途分类
i-rs-debt add "消费贷" --debt-type loan --amount 10000 --tags consumption,digital

# 按优先级分类
i-rs-debt add "高息债务" --debt-type credit_card --amount 5000 --tags high-interest,priority

# 列出所有高息债务
i-rs-debt list --tag high-interest
```

---

## 完整工作流示例

### 月度债务管理

```bash
# 1. 月初查看债务状态
i-rs-debt list

# 2. 查看本月到期债务
i-rs-debt stats

# 3. 记录还款
i-rs-debt pay "信用卡A" --amount 1000 --note "5月还款"
i-rs-debt pay "车贷" --amount 3500 --note "5月月供"

# 4. 检查是否有逾期
i-rs-debt list --overdue

# 5. 更新到期日（如果延期）
i-rs-debt update "朋友借款" --due-date 2026-07-01
```

### 年度总结

```bash
# 1. 统计全年债务
i-rs-debt stats --by-type

# 2. 查看已还清的债务
i-rs-debt list --paid-off

# 3. 查看剩余债务
i-rs-debt list

# 4. 规划新一年的还款计划
# (使用 stats 数据制定预算)
```

---

## JSON 输出集成

### 脚本中使用

```bash
# 获取债务列表
i-rs-debt list --json > debts.json

# 获取统计数据
i-rs-debt stats --by-type --json > stats.json

# 获取特定债务详情
i-rs-debt get "信用卡A" --payments --json > card_detail.json
```

### 在其他工具中使用

```bash
# 配合 jq 使用
i-rs-debt stats --json | jq '.data.total_remaining'

# 导出债务列表
i-rs-debt list --json | jq '.data[].name'

# 统计逾期债务数量
i-rs-debt list --json | jq '.data | map(select(.is_overdue)) | length'
```
