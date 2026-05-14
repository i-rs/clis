# i-rs-budget 使用示例

## 基础使用

### 设置日常预算

```bash
# 创建餐饮月度预算
i-rs-budget add food 500

# 记录餐饮支出
i-rs-budget expense food 25.50 --description "午餐"
i-rs-budget expense food 15.00 --description "咖啡"
i-rs-budget expense food 45.00 --description "超市购物"

# 查看本月统计
i-rs-budget stats
```

### 管理多个预算类别

```bash
# 添加多个预算
i-rs-budget add groceries 800 --period monthly
i-rs-budget add entertainment 300 --period monthly
i-rs-budget add transportation 200 --period monthly
i-rs-budget add rent 3000 --period monthly
i-rs-budget add utilities 150 --period monthly

# 查看所有预算
i-rs-budget list budgets
```

## 周期管理

### 周预算

```bash
# 设置周预算
i-rs-budget add groceries 300 --period weekly
i-rs-budget add entertainment 100 --period weekly

# 查看本周统计
i-rs-budget stats --period weekly
```

### 年预算

```bash
# 设置年度预算（如旅行）
i-rs-budget add vacation 10000 --period yearly

# 查看年度统计
i-rs-budget stats --period yearly
```

## 支出记录

### 日常支出记录

```bash
# 餐饮支出
i-rs-budget expense food 35.00 --description "午餐"
i-rs-budget expense food 20.00 --description "下午茶"
i-rs-budget expense food 80.00 --description "晚餐"

# 购物支出
i-rs-budget expense groceries 150.50 --description "超市"
i-rs-budget expense groceries 45.30 --description "便利店"

# 交通支出
i-rs-budget expense transportation 5.00 --description "地铁"
i-rs-budget expense transportation 30.00 --description "打车"

# 娱乐支出
i-rs-budget expense entertainment 60.00 --description "电影票"
i-rs-budget expense entertainment 120.00 --description "游戏"
```

### 带标签的支出

```bash
# 使用标签分类支出
i-rs-budget expense food 88.00 --description "朋友聚餐" --tags 社交,周末
i-rs-budget expense entertainment 200.00 --description "演唱会门票" --tags 音乐,特殊活动
i-rs-budget expense groceries 300.00 --description "周末大采购" --tags 周末,家庭
```

### 指定日期的支出

```bash
# 记录过去的支出
i-rs-budget expense food 45.00 --description "午餐" --date 2024-01-10
i-rs-budget expense groceries 120.00 --description "超市" --date 2024-01-12

# 记录未来预算（如预订）
i-rs-budget expense entertainment 500.00 --description "酒店预订" --date 2024-03-15
```

## 统计与分析

### 查看月度统计

```bash
# 默认显示本月统计
i-rs-budget stats

# 指定月份
i-rs-budget stats --period monthly
```

### 类别统计

```bash
# 查看特定类别统计
i-rs-budget stats --category food
i-rs-budget stats --category entertainment

# 特定类别 + 特定周期
i-rs-budget stats --category food --period weekly
```

### 查看支出记录

```bash
# 列出所有支出
i-rs-budget list expenses

# 按类别列出支出
i-rs-budget list expenses --category food
i-rs-budget list expenses --category entertainment
```

## 预算管理

### 更新预算

```bash
# 增加预算金额
i-rs-budget update food --amount 600

# 调整周期
i-rs-budget update groceries --period monthly

# 更新标签
i-rs-budget update entertainment --tags fun,周末
```

### 删除操作

```bash
# 删除整个预算类别（同时删除相关支出）
i-rs-budget delete --category entertainment

# 删除单笔支出
i-rs-budget delete --expense-id abc12345

# 查看支出 ID
i-rs-budget list expenses
```

## JSON 输出

所有命令支持 JSON 输出，便于程序调用。

```bash
# JSON 格式列出预算
i-rs-budget list budgets --json

# JSON 格式列出支出
i-rs-budget list expenses --json

# JSON 格式统计
i-rs-budget stats --json

# JSON 格式获取详情
i-rs-budget get --category food --json

# JSON 格式获取单笔支出
i-rs-budget get --expense-id abc12345 --json
```

## 完整工作流示例

### 月度预算追踪

```bash
# 1. 月初设置预算
i-rs-budget add food 1000 --period monthly
i-rs-budget add groceries 500 --period monthly
i-rs-budget add entertainment 300 --period monthly
i-rs-budget add transportation 200 --period monthly

# 2. 日常记录支出
i-rs-budget expense food 35.00 --description "午餐"
i-rs-budget expense food 25.00 --description "咖啡"
i-rs-budget expense groceries 150.00 --description "超市"
i-rs-budget expense transportation 10.00 --description "地铁"

# 3. 定期查看统计
i-rs-budget stats

# 4. 根据统计调整
i-rs-budget update food --amount 1200  # 如果超支
```

### 周预算追踪

```bash
# 1. 设置周预算
i-rs-budget add groceries 300 --period weekly
i-rs-budget add entertainment 100 --period weekly

# 2. 记录周内支出
i-rs-budget expense groceries 80.00 --description "周一采购"
i-rs-budget expense entertainment 50.00 --description "电影"
i-rs-budget expense groceries 120.00 --description "周三补货"

# 3. 周中查看进度
i-rs-budget stats --period weekly

# 4. 决定是否调整
# 如果预算充足，可以适当放松
# 如果接近超支，控制消费
```

## 数据导出（通过 JSON）

```bash
# 导出所有数据
i-rs-budget list budgets --json > budgets.json
i-rs-budget list expenses --json > expenses.json

# 导出统计
i-rs-budget stats --json > stats.json
```
