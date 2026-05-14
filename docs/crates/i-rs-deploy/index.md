# i-rs-deploy

部署记录 CLI - 跟踪部署、管理回滚、查看统计数据。

## 功能特点

- 跨项目和环境的部署跟踪
- 部署状态记录（成功/失败/回滚中/已回滚）
- 回滚记录管理
- 统计数据和部署时间线
- 标签和备注支持

## 安装

```bash
npm install -g @i-rs/i-rs-deploy
# or
brew install i-rs/homebrew-tap/i-rs-deploy
```

## 快速开始

```bash
# 添加部署记录
i-rs-deploy add myapp production v1.2.3 --status success

# 列出所有部署
i-rs-deploy list

# 查看部署详情
i-rs-deploy get abc12345

# 回滚到上一版本
i-rs-deploy rollback myapp production

# 查看统计信息
i-rs-deploy stats
```

## 数据存储

- macOS: `~/.config/i-rs/deploy.json`
- Linux: `~/.config/i-rs/deploy.json`
- Windows: `~\AppData\Roaming\i-rs\deploy.json`

## 许可证

MIT OR Apache-2.0
