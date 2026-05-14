# i-rs-read 使用指南

## add - 添加书籍

添加一本新书到阅读列表。

### 命令格式

```bash
i-rs-read add <书名> <作者> <总页数> [选项]
```

### 参数说明

| 参数 | 说明 | 必填 |
|------|------|------|
| `书名` | 书籍名称 | 是 |
| `作者` | 作者姓名 | 是 |
| `总页数` | 书籍总页数 | 是 |

### 选项

| 选项 | 说明 |
|------|------|
| `--tags <标签>` | 添加标签，可多次指定 |
| `--remark <备注>` | 添加备注，可多次指定 |

### 示例

```bash
# 基本用法
i-rs-read add "Rust 编程之道" "Steve Klabnik" 500

# 添加标签
i-rs-read add "Programming Rust" "Jim Blandy" 400 --tags rust,programming

# 添加标签和备注
i-rs-read add "Clean Code" "Robert C. Martin" 464 --tags programming,best-practices --remark "必读经典"
```

---

## list - 列出书籍

列出所有书籍或按条件筛选。

### 命令格式

```bash
i-rs-read list [选项]
```

### 选项

| 选项 | 说明 |
|------|------|
| `--tag <标签>` | 按标签筛选书籍 |
| `--status <状态>` | 按阅读状态筛选 |

### 阅读状态

- `reading` - 阅读中
- `completed` - 已完成
- `paused` - 暂停
- `dropped` - 放弃
- `to_read` - 待读

### 示例

```bash
# 列出所有书籍
i-rs-read list

# 按标签筛选
i-rs-read list --tag rust

# 按状态筛选
i-rs-read list --status reading

# 组合筛选
i-rs-read list --tag programming --status completed
```

---

## get - 获取书籍详情

查看单本书籍的详细信息。

### 命令格式

```bash
i-rs-read get <书名>
```

### 示例

```bash
# 获取书籍详情
i-rs-read get "Rust 编程之道"

# JSON 格式输出
i-rs-read get "Rust 编程之道" --json
```

---

## update - 更新书籍

更新书籍的阅读进度或信息。

### 命令格式

```bash
i-rs-read update <书名> [选项]
```

### 选项

| 选项 | 说明 |
|------|------|
| `--current-page <页码>` | 更新当前阅读页数 |
| `--status <状态>` | 更新阅读状态 |
| `--rating <评分>` | 添加或修改评分（0-5） |
| `--review <评论>` | 添加或修改评论 |
| `--tags <标签>` | 更新标签（逗号分隔） |
| `--add-remark <备注>` | 添加备注 |
| `--remove-remark <索引>` | 删除备注（1-based 索引） |

### 示例

```bash
# 更新阅读进度
i-rs-read update "Rust 编程之道" --current-page 250

# 更新状态
i-rs-read update "Rust 编程之道" --status reading

# 标记为已完成
i-rs-read update "Rust 编程之道" --status completed

# 完成后添加评分
i-rs-read update "Rust 编程之道" --status completed --rating 5

# 添加评论
i-rs-read update "Rust 编程之道" --review "非常棒的 Rust 入门书籍！"

# 更新标签
i-rs-read update "Rust 编程之道" --tags rust,programming

# 添加备注
i-rs-read update "Rust 编程之道" --add-remark "第5章并发部分很有挑战"

# 删除备注
i-rs-read update "Rust 编程之道" --remove-remark 1
```

### 注意事项

- 当前页数不能超过总页数
- 评分范围为 0-5
- 当状态设为 `completed` 时，当前页数会自动更新为总页数

---

## delete - 删除书籍

从阅读列表中删除一本书。

### 命令格式

```bash
i-rs-read delete <书名>
```

### 示例

```bash
# 删除书籍
i-rs-read delete "旧书名"
```

---

## stats - 阅读统计

查看阅读统计数据。

### 命令格式

```bash
i-rs-read stats [选项]
```

### 选项

| 选项 | 说明 |
|------|------|
| `--tag <标签>` | 按标签筛选统计 |

### 统计内容

- 总书籍数量
- 总页数
- 已读页数
- 各种状态的数量（阅读中、已完成、待读、暂停、放弃）
- 已评分书籍数量
- 平均评分

### 示例

```bash
# 查看全部统计
i-rs-read stats

# 按标签筛选统计
i-rs-read stats --tag rust

# JSON 格式输出
i-rs-read stats --json
```

---

## example - 使用示例

显示详细的使用示例。

### 命令格式

```bash
i-rs-read example
```

### 示例

```bash
# 查看所有使用示例
i-rs-read example
```

---

## skill - AI 技能文档

查看 AI 技能文档，用于 AI 助手理解如何使用此工具。

### 命令格式

```bash
i-rs-read skill [选项]
```

### 选项

| 选项 | 说明 |
|------|------|
| `--summary` | 仅显示摘要 |
| `--content` | 显示完整内容 |

### 示例

```bash
# 查看完整技能文档
i-rs-read skill

# 仅显示摘要
i-rs-read skill --summary

# 显示内容部分
i-rs-read skill --content
```

---

## 全局选项

| 选项 | 说明 |
|------|------|
| `--json` | 以 JSON 格式输出结果 |

### 示例

```bash
# JSON 格式列出书籍
i-rs-read list --json

# JSON 格式获取书籍详情
i-rs-read get "书名" --json

# JSON 格式查看统计
i-rs-read stats --json
```
