# i-rs-quote

语录收藏 CLI 工具 - 用于收集、整理和展示名人名言、书籍摘录等。

## 功能特点

- 语录内容管理（内容、作者、出处）
- 标签分类系统
- 按作者搜索
- 随机展示语录
- 个人备注支持
- JSON 输出支持

## 安装

```bash
npm install -g @i-rs/i-rs-quote
# 或者
brew install i-rs/homebrew-tap/i-rs-quote
```

## 快速开始

```bash
# 添加语录
i-rs-quote add --content "The only way to do great work is to love what you do." --author "Steve Jobs"

# 按作者搜索
i-rs-quote list --author "Steve"

# 列出所有语录
i-rs-quote list

# 随机展示
i-rs-quote random

# 查看详情
i-rs-quote get <uuid>

# 删除语录
i-rs-quote delete <uuid>
```

## 数据存储

- macOS: `~/.config/i-rs/quotes.json`
- Linux: `~/.config/i-rs/quotes.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## 命令详解

### add - 添加语录

```bash
i-rs-quote add --content "语录内容" [--author "作者"] [--source "出处"] [--tag 标签] [--remark "备注"]
```

### list - 列出语录

```bash
i-rs-quote list [--tag 标签] [--author 作者]
```

### get - 查看详情

```bash
i-rs-quote get <ID>
```

### random - 随机展示

```bash
i-rs-quote random
```

### delete - 删除语录

```bash
i-rs-quote delete <ID>
```

## JSON 输出

所有命令支持 `--json` 全局标志：

```bash
i-rs-quote list --json
i-rs-quote get <ID> --json
```

## 许可证

MIT OR Apache-2.0
