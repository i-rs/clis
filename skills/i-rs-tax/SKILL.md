---
name: "i-rs-tax"
description: "税务记录管理 CLI 工具。用于记录个人所得税、增值税等税务信息，支持年度统计和报税状态跟踪。"
---

# i-rs-tax 税务记录管理

## Global Flags

- `--json` — Output in JSON format

税务记录管理 CLI 工具，用于记录个人所得税、增值税等税务信息，支持年度统计和报税状态跟踪。

## 存储

- 配置: `~/.config/i-rs/tax.json`
- 可通过环境变量 `CONFIG_DIR` 覆盖

## 税种类型

- `personal` / `个人所得税`: 个人所得税
- `vat` / `增值税`: 增值税

## 报税状态

- `unreported` / `未申报`: 尚未申报
- `filing` / `申报中`: 申报中
- `filed` / `已申报`: 已申报
- `paid` / `已缴纳`: 已缴纳

## 命令

### add - 添加记录
```bash
i-rs-tax add <名称> --tax-type <税种> --amount <金额> --date <日期> [选项]
```

选项:
- `--tax-type, -t`: 税种类型 (personal/vat)
- `--amount, -a`: 金额
- `--date, -d`: 日期 (YYYY-MM-DD)
- `--year, -y`: 年度 (可选，默认从日期提取)
- `--status, -s`: 报税状态 (可选，默认 unreported)
- `--tag`: 标签 (可多次指定)
- `--remark`: 备注 (可多次指定)

示例:
```bash
i-rs-tax add 个税2024 -t personal -a 12000 -d 2024-03-15 -s filed --tag 工资
```

### list - 列出记录
```bash
i-rs-tax list [选项]
```

选项:
- `--tag, -t`: 按标签过滤
- `--year, -y`: 按年度过滤
- `--tax-type`: 按税种过滤 (personal/vat)

### get - 查看详情
```bash
i-rs-tax get <名称>
```

### delete - 删除记录
```bash
i-rs-tax delete <名称>
```

### stats - 年度统计
```bash
i-rs-tax stats [选项]
```

选项:
- `--year, -y`: 指定年度 (默认当前年度)
- `--tax-type, -t`: 按税种过滤

### example - 使用示例
```bash
i-rs-tax example
```

### skill - AI 技能文档
```bash
i-rs-tax skill [summary]
```

## 数据结构

```json
{
  "name": "个税2024",
  "tax_type": "个人所得税",
  "amount": 12000.00,
  "date": "2024-03-15",
  "year": 2024,
  "status": "已申报",
  "tags": ["工资"],
  "remark": [],
  "created_at": "2024-03-15T10:00:00Z",
  "updated_at": "2024-03-15T10:00:00Z"
}
```

## 典型使用场景

1. **月度个税申报**: 记录每月工资个税，跟踪申报状态
2. **季度增值税申报**: 记录季度增值税，便于年度汇总
3. **年终奖税务**: 单独记录年终奖税务信息
4. **年度汇总**: 使用 stats 命令统计年度税务情况

### data

Manage data (export, import, clear).

```bash
i-rs-tax data export
i-rs-tax data import [FILE]
i-rs-tax data clear
```

## Examples

```bash
# JSON output

i-rs-tax list --json
```
