# i-rs-budget

个人预算管理 CLI 工具，支持预算设置、支出记录和统计分析。

## 功能特性

- **预算管理**: 创建、更新、删除不同周期的预算（每日/每周/每月/每年）
- **支出记录**: 记录每笔支出，包含金额、描述、日期和标签
- **统计报表**: 查看预算使用情况，了解已花费、剩余金额和完成百分比
- **分类统计**: 按类别查看支出明细
- **周期筛选**: 支持按日/周/月/年查看统计数据
- **JSON 输出**: 支持 `--json` 全局标志，便于程序调用

## 安装

```bash
cargo install i-rs-budget
# 或
brew install i-rs/homebrew-tap/i-rs-budget
```

## 快速开始

### 添加预算

```bash
# 添加月度预算（默认）
i-rs-budget add food 500

# 添加不同周期的预算
i-rs-budget add groceries 300 --period weekly
i-rs-budget add rent 2000 --period monthly
i-rs-budget add vacation 5000 --period yearly

# 添加带标签的预算
i-rs-budget add entertainment 200 --tags fun,leisure
```

### 记录支出

```bash
# 记录支出
i-rs-budget expense food 25.50 --description "午餐"
i-rs-budget expense groceries 120.30 --date 2024-01-15

# 带标签记录
i-rs-budget expense entertainment 50 --description "电影票" --tags movie
```

### 查看预算和支出

```bash
# 列出所有预算
i-rs-budget list budgets

# 列出所有支出
i-rs-budget list expenses

# 按类别查看
i-rs-budget list expenses --category food
```

### 查看统计

```bash
# 查看本月统计
i-rs-budget stats

# 按周期查看
i-rs-budget stats --period weekly
i-rs-budget stats --period yearly

# 按类别查看
i-rs-budget stats --category food
```

### 更新和删除

```bash
# 更新预算
i-rs-budget update food --amount 600

# 删除预算
i-rs-budget delete --category food

# 删除单笔支出
i-rs-budget delete --expense-id abc12345
```

## 数据存储

- macOS: `~/.config/i-rs/budget.json`
- Linux: `~/.config/i-rs/budget.json`
- Windows: `~\AppData\Roaming\i-rs\budget.json`

可通过 `CONFIG_DIR` 环境变量覆盖默认路径。

## 许可证

MIT OR Apache-2.0
