# i-rs-tax

税务记录管理 CLI 工具，用于记录个人所得税、增值税等税务信息。

## 功能特性

- 税务记录管理（个税/增值税）
- 金额和日期追踪
- 报税状态管理
- 标签分类
- 年度统计
- JSON 输出支持

## 安装

```bash
cargo install i-rs-tax
# 或
brew install i-rs/homebrew-tap/i-rs-tax
```

## 快速开始

### 添加税务记录

```bash
# 添加个人所得税
i-rs-tax add 个税2024 --tax-type personal --amount 12000 --date 2024-03-15

# 添加增值税记录
i-rs-tax add 增值税Q1 --tax-type vat --amount 5000 --date 2024-04-01 --status filed
```

### 查看记录

```bash
# 列出所有记录
i-rs-tax list

# 按年度筛选
i-rs-tax list --year 2024

# 按标签筛选
i-rs-tax list --tag 工资

# 按税种筛选
i-rs-tax list --tax-type personal
```

### 查看详情

```bash
i-rs-tax get 个税2024
```

### 删除记录

```bash
i-rs-tax delete 个税2024
```

### 年度统计

```bash
# 当前年度统计
i-rs-tax stats

# 指定年度统计
i-rs-tax stats --year 2024
```

## 税种类型

- `personal` / `个人所得税`: 个人所得税
- `vat` / `增值税`: 增值税

## 报税状态

- `unreported` / `未申报`: 尚未申报
- `filing` / `申报中`: 申报中
- `filed` / `已申报`: 已申报
- `paid` / `已缴纳`: 已缴纳

## 数据存储

- macOS: `~/.config/i-rs/tax.json`
- Linux: `~/.config/i-rs/tax.json`
- Windows: `~\AppData\Roaming\i-rs\tax.json`

可通过环境变量 `CONFIG_DIR` 覆盖配置目录。

## JSON 输出

所有命令支持 `--json` 标志：

```bash
i-rs-tax list --json
i-rs-tax get 个税2024 --json
i-rs-tax stats --year 2024 --json
```

## 示例

```bash
# 添加完整的税务记录
i-rs-tax add 个税2024 \
  --tax-type personal \
  --amount 12000 \
  --date 2024-03-15 \
  --status filed \
  --tag 工资 \
  --tag 年终奖 \
  --remark 年终奖申报

# 查看所有个人所得税记录
i-rs-tax list --tax-type personal

# 统计 2024 年度税务
i-rs-tax stats --year 2024
```

## License

MIT OR Apache-2.0
