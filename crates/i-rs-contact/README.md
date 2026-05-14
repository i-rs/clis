# i-rs-contact

联系人管理 CLI 工具 - 管理联系人、跟踪关系和联系记录。

## 功能

- 联系人基本信息（姓名、电话、邮箱、关系）
- 标签分类（朋友、同事、家人等）
- 最近联系记录
- 联系频率统计
- 提醒功能（30天以上未联系）

## 安装

```bash
cargo build -p i-rs-contact
# 或
cargo install --path crates/i-rs-contact
```

## 快速开始

```bash
# 添加联系人
i-rs-contact add John --phone 13800138000 --email john@example.com --relationship friend

# 添加带标签的联系人
i-rs-contact add Alice --phone 13900139000 --tag family --tag important

# 列出所有联系人
i-rs-contact list

# 按标签筛选
i-rs-contact list --tag family

# 查看联系人详情
i-rs-contact get John

# 更新联系人
i-rs-contact update John --phone 13800138001

# 删除联系人
i-rs-contact delete old_contact

# 查看统计
i-rs-contact stats

# 提醒联系
i-rs-contact remind

# 自定义提醒天数
i-rs-contact remind --days 7
```

## 数据存储

- macOS: `~/.config/i-rs/contacts.json`
- Linux: `~/.config/i-rs/contacts.json`
- Windows: `~\AppData\Roaming\i-rs\contacts.json`

## 许可

MIT OR Apache-2.0
