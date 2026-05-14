# i-rs-tax 使用示例

## 基础操作

### 添加个人所得税记录

```bash
i-rs-tax add 个人所得税2024 \
  --tax-type personal \
  --amount 12000 \
  --date 2024-03-15 \
  --status filed
```

### 添加增值税记录

```bash
i-rs-tax add 增值税Q1 \
  --tax-type vat \
  --amount 5000 \
  --date 2024-04-01 \
  --status paid
```

### 带标签和备注的记录

```bash
i-rs-tax add 年终奖个税 \
  --tax-type personal \
  --amount 8000 \
  --date 2024-01-20 \
  --status filed \
  --tag 工资 \
  --tag 年终奖 \
  --remark 2023年度年终奖申报
```

## 记录管理

### 查看所有记录

```bash
i-rs-tax list
```

### 按年度查看

```bash
i-rs-tax list --year 2024
```

### 按标签筛选

```bash
# 包含"工资"标签的记录
i-rs-tax list --tag 工资

# 包含"年终奖"标签的记录
i-rs-tax list --tag 年终奖
```

### 按税种筛选

```bash
# 仅个人所得税
i-rs-tax list --tax-type personal

# 仅增值税
i-rs-tax list --tax-type vat
```

### 查看记录详情

```bash
i-rs-tax get 个人所得税2024
```

### 删除记录

```bash
i-rs-tax delete 个人所得税2024
```

## 年度统计

### 查看当前年度统计

```bash
i-rs-tax stats
```

### 查看指定年度统计

```bash
i-rs-tax stats --year 2024
```

### 按税种统计

```bash
# 仅个人所得税统计
i-rs-tax stats --tax-type personal

# 仅增值税统计
i-rs-tax stats --tax-type vat
```

## JSON 输出

所有命令都支持 `--json` 标志：

```bash
# JSON 列表
i-rs-tax list --json

# JSON 详情
i-rs-tax get 个人所得税2024 --json

# JSON 统计
i-rs-tax stats --year 2024 --json
```

## 工作流程示例

### 月度报税流程

```bash
# 1. 添加本月个人所得税
i-rs-tax add 个税4月 \
  --tax-type personal \
  --amount 3500 \
  --date 2024-04-30 \
  --status unreported \
  --tag 月度申报

# 2. 申报后更新状态
i-rs-tax add 个税4月 \
  --tax-type personal \
  --amount 3500 \
  --date 2024-04-30 \
  --status filed \
  --tag 月度申报

# 3. 缴纳后更新状态
i-rs-tax add 个税4月 \
  --tax-type personal \
  --amount 3500 \
  --date 2024-04-30 \
  --status paid \
  --tag 月度申报

# 4. 查看统计
i-rs-tax stats --year 2024
```

### 季度增值税申报

```bash
# Q1 增值税
i-rs-tax add 增值税Q1 \
  --tax-type vat \
  --amount 15000 \
  --date 2024-04-15 \
  --status filed \
  --tag 季度申报

# Q2 增值税
i-rs-tax add 增值税Q2 \
  --tax-type vat \
  --amount 18000 \
  --date 2024-07-15 \
  --status filed \
  --tag 季度申报

# 查看年度增值税统计
i-rs-tax stats --year 2024 --tax-type vat
```

### 年终奖税务处理

```bash
# 记录年终奖个税
i-rs-tax add 年终奖个税2024 \
  --tax-type personal \
  --amount 12000 \
  --date 2024-01-31 \
  --status paid \
  --tag 年终奖 \
  --tag 工资 \
  --remark 2023年度年终奖

# 查看年终奖相关税务
i-rs-tax list --tag 年终奖
```

## 常见问题

### Q: 如何查看所有未申报的记录？

```bash
# 查看所有记录，然后筛选
i-rs-tax list | grep 未申报

# 或者按状态记录
i-rs-tax list --tag 未申报
```

### Q: 如何导出年度数据？

```bash
# 导出为 JSON
i-rs-tax list --year 2024 --json > tax-2024.json
```

### Q: 如何统计某类型的总税额？

```bash
# 个人所得税总额
i-rs-tax stats --year 2024 --tax-type personal

# 增值税总额
i-rs-tax stats --year 2024 --tax-type vat
```
