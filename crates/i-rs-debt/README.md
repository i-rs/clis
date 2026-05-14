# i-rs-debt

债务管理 CLI 工具，用于追踪信用卡债务、贷款和借款，支持还款记录和逾期提醒。

## 功能特点

- 支持多种债务类型：信用卡、贷款、借款
- 记录还款历史
- 分期还款追踪
- 逾期提醒
- 标签支持
- 统计数据视图
- JSON 输出支持

## 安装

```bash
npm install -g @i-rs/i-rs-debt
# 或
brew install i-rs/homebrew-tap/i-rs-debt
```

## 快速开始

```bash
# 添加信用卡债务
i-rs-debt add "信用卡A" --debt-type credit_card --amount 10000 --interest-rate 15.0

# 添加贷款
i-rs-debt add "车贷" --debt-type loan --amount 50000 --tags car,vehicle

# 列出所有债务
i-rs-debt list

# 记录还款
i-rs-debt pay "信用卡A" --amount 500

# 查看债务详情
i-rs-debt get "信用卡A"

# 查看统计
i-rs-debt stats
```

## 数据存储

- macOS: `~/.config/i-rs/debt.json`
- Linux: `~/.config/i-rs/debt.json`
- Windows: `~\AppData\Roaming\i-rs\debt.json`

## 许可

MIT OR Apache-2.0
