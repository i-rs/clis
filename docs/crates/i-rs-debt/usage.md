# i-rs-debt 详细命令参考

## Global Flags

- `--json` — Output in JSON format

## 债务类型

i-rs-debt 支持三种债务类型：

| 类型 | 说明 | 别名 |
|------|------|------|
| credit_card | 信用卡债务 | cc |
| loan | 贷款 | l |
| borrowed | 借款 | b |

## 命令详解

### add 命令

添加新债务：

```bash
# 基本用法
i-rs-debt add "信用卡A" --debt-type credit_card --amount 10000

# 带利率和到期日
i-rs-debt add "车贷" \
  --debt-type loan \
  --amount 50000 \
  --interest-rate 5.5 \
  --due-date 2026-12-31

# 带标签和备注
i-rs-debt add "朋友借款" \
  --debt-type borrowed \
  --amount 2000 \
  --tags personal \
  --remark "向张三借款" \
  --remark "计划6个月还清"
```

### list 命令

列出债务，支持多种筛选：

```bash
# 列出所有债务
i-rs-debt list

# 按标签筛选
i-rs-debt list --tag car
i-rs-debt list --tag "high-priority"

# 只显示逾期债务
i-rs-debt list --overdue

# 只显示已还清债务
i-rs-debt list --paid-off
```

### get 命令

查看债务详情：

```bash
# 基本信息
i-rs-debt get "信用卡A"

# 包含还款历史
i-rs-debt get "信用卡A" --payments
```

### pay 命令

记录还款：

```bash
# 基本还款
i-rs-debt pay "信用卡A" --amount 500

# 带备注
i-rs-debt pay "信用卡A" --amount 1000 --note "最低还款额"
```

### update 命令

更新债务信息：

```bash
# 更新利率
i-rs-debt update "信用卡A" --interest-rate 12.0

# 重命名
i-rs-debt update "旧名称" --rename "新名称"

# 更新总额
i-rs-debt update "信用卡A" --amount 15000

# 添加标签
i-rs-debt update "信用卡A" --add-tags important,urgent

# 移除标签
i-rs-debt update "信用卡A" --remove-tags old-tag

# 更新到期日
i-rs-debt update "信用卡A" --due-date 2026-07-01
```

### delete 命令

删除债务：

```bash
# 带确认
i-rs-debt delete "旧债务"

# 跳过确认
i-rs-debt delete "旧债务" --force
```

### stats 命令

查看统计：

```bash
# 总体统计
i-rs-debt stats

# 按类型分组
i-rs-debt stats --by-type
```

## JSON 输出

所有命令支持 `--json` 参数：

```bash
i-rs-debt list --json
i-rs-debt get "信用卡A" --json
i-rs-debt stats --json
```

**JSON 响应格式:**

列表响应:
```json
{
  "success": true,
  "data": [...],
  "meta": {
    "count": 10,
    "filter": "work"
  }
}
```

详情响应:
```json
{
  "success": true,
  "data": {...}
}
```

错误响应:
```json
{
  "success": false,
  "error": {
    "code": "NOT_FOUND",
    "message": "Debt 'xxx' not found"
  }
}
```

### data

Manage data (export, import, clear).

```bash
i-rs-debt data export
i-rs-debt data import [FILE]
i-rs-debt data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
### example

Show usage examples.

```bash
i-rs-debt example
```
### skill

Show skill information.

```bash
i-rs-debt skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/debt.json`
- Linux: `~/.config/i-rs/debt.json`
- Windows: `~\AppData\Roaming\i-rs\debt.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-debt list
```
