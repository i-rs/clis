# i-rs-quote

语录收藏 CLI 工具 - 用于收集、整理和展示名人名言、书籍摘录等。

## 简介

i-rs-quote 是一个轻量级的命令行工具，帮助你收藏和管理有意义的语录、名言警句、书籍摘录等。

## 主要功能

- **语录管理**: 添加、查看、删除语录
- **作者信息**: 记录语录作者和出处
- **标签分类**: 使用标签组织语录
- **按作者搜索**: 快速找到特定作者的语录
- **随机展示**: 从收藏中随机展示一条语录
- **备注功能**: 为语录添加个人注释

## 快速开始

```bash
# 添加第一条语录
i-rs-quote add --content "The only way to do great work is to love what you do." --author "Steve Jobs"

# 随机展示一条语录
i-rs-quote random

# 列出所有语录
i-rs-quote list

# 按标签筛选
i-rs-quote list --tag inspiration

# 按作者搜索
i-rs-quote list --author "Steve"
```

## 数据存储

- 配置文件: `~/.config/i-rs/quotes.json`
- 支持通过 `CONFIG_DIR` 环境变量自定义存储位置
