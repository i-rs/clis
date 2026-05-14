# i-rs-read 使用示例

## 基础操作

### 添加第一本书

```bash
# 添加一本简单的技术书籍
i-rs-read add "Rust 编程之道" "Steve Klabnik" 500

# 输出
# Book Added
# ✓ Name: Rust 编程之道
# Author: Steve Klabnik
# Total Pages: 500
# Status: ToRead
```

### 添加带标签的书籍

```bash
# 添加多本同主题书籍
i-rs-read add "Programming Rust" "Jim Blandy" 400 --tags rust,programming
i-rs-read add "The Rustonomicon" "Nicholas Matsakis" 300 --tags rust,programming,advanced

# 添加非技术书籍
i-rs-read add "人类简史" "尤瓦尔·赫拉利" 450 --tags history,non-fiction
```

### 添加书籍时添加备注

```bash
i-rs-read add "Clean Code" "Robert C. Martin" 464 \
  --tags programming,best-practices \
  --remark "同事推荐的经典之作" \
  --remark "需要反复阅读"
```

---

## 阅读进度追踪

### 开始阅读一本书

```bash
# 假设这本书之前状态是 to_read，现在开始读
i-rs-read update "Rust 编程之道" --status reading

# 每次阅读后更新进度
i-rs-read update "Rust 编程之道" --current-page 50
i-rs-read update "Rust 编程之道" --current-page 100
i-rs-read update "Rust 编程之道" --current-page 150
```

### 查看进度

```bash
# 列出所有正在阅读的书籍
i-rs-read list --status reading

# 输出示例
# Books
# ┌──────────────────┬────────────────┬────────┬──────────┬──────────┬────────┐
# │ name             │ author        │ pages  │ status   │ progress │ rating │
# ├──────────────────┼────────────────┼────────┼──────────┼──────────┼────────┤
# │ Rust 编程之道     │ Steve Klabnik │ 150/500│ Reading  │ 30.0%    │ -      │
# └──────────────────┴────────────────┴────────┴──────────┴──────────┴────────┘
#
# Total: 1 books
```

### 完成一本书

```bash
# 标记为已完成
i-rs-read update "Rust 编程之道" --status completed

# 同时添加评分
i-rs-read update "Rust 编程之道" --status completed --rating 5

# 添加详细评论
i-rs-read update "Rust 编程之道" --status completed --rating 5 --review "这是我读过最好的 Rust 入门书籍，讲解深入浅出，示例丰富！"
```

---

## 标签管理

### 按标签筛选书籍

```bash
# 查看所有 Rust 相关书籍
i-rs-read list --tag rust

# 查看所有待读书籍
i-rs-read list --status to_read --tag non-fiction

# 查看按标签统计
i-rs-read stats --tag programming
```

### 更新书籍标签

```bash
# 更新一本书的标签
i-rs-read update "Rust 编程之道" --tags rust,programming,beginner-friendly

# 清除后重新设置
i-rs-read update "Rust 编程之道" --tags rust,programming
```

---

## 备注管理

### 添加备注

```bash
# 记录阅读时的想法
i-rs-read update "Rust 编程之道" --add-remark "第3章的所有权概念终于理解了"
i-rs-read update "Rust 编程之道" --add-remark "第5章生命周期有点难，需要再看看"
i-rs-read update "Rust 编程之道" --add-remark "完成了所有权章节！"
```

### 删除备注

```bash
# 查看书籍详情，确认备注索引
i-rs-read get "Rust 编程之道"

# 删除错误的备注
i-rs-read update "Rust 编程之道" --remove-remark 2
```

---

## 阅读统计

### 查看整体统计

```bash
i-rs-read stats

# 输出示例
# Reading Statistics
#
# 📚 Overall:
#   Total books: 10
#   Total pages: 5000
#   Pages read: 2150/5000 (43.0%)
#
# 📖 Status:
#   Reading: 2
#   Completed: 5
#   To Read: 2
#   Paused: 1
#   Dropped: 0
#
# ⭐ Rating:
#   Rated books: 4
#   Average rating: 4.50
```

### 按标签统计

```bash
# 只统计 Rust 相关书籍
i-rs-read stats --tag rust

# 只统计技术类书籍
i-rs-read stats --tag programming
```

---

## 整理阅读列表

### 批量添加到待读清单

```bash
# 添加多本书到待读清单
i-rs-read add "设计模式" "Gang of Four" 395 --tags programming
i-rs-read add "重构" "Martin Fowler" 431 --tags programming,refactoring
i-rs-read add "算法导论" "Thomas H. Cormen" 1312 --tags algorithm,programming
```

### 暂停一本书

```bash
# 当前不想读，先暂停
i-rs-read update "算法导论" --status paused
```

### 放弃一本书

```bash
# 确实不适合自己，标记为放弃
i-rs-read update "算法导论" --status dropped
```

### 查看所有状态

```bash
# 查看各类状态的书籍
i-rs-read list --status reading    # 正在读
i-rs-read list --status completed  # 已完成
i-rs-read list --status paused     # 暂停
i-rs-read list --status dropped    # 放弃
i-rs-read list --status to_read    # 待读
```

---

## 数据查询

### 获取书籍详情

```bash
# 查看完整书籍信息
i-rs-read get "Rust 编程之道"
```

### JSON 输出

```bash
# 获取 JSON 格式数据
i-rs-read get "Rust 编程之道" --json

# 列出 JSON 格式书籍
i-rs-read list --json

# 统计 JSON 格式
i-rs-read stats --json
```

---

## 删除书籍

### 删除一本书

```bash
# 删除不需要的书籍
i-rs-read delete "旧书名"
```

---

## 完整工作流示例

### 从添加到完成的完整流程

```bash
# 1. 添加书籍
i-rs-read add "深入理解计算机系统" "Randal E. Bryant" 879 \
  --tags cs,programming,systems \
  --remark "技术书籍堆中的一本"

# 2. 开始阅读
i-rs-read update "深入理解计算机系统" --status reading

# 3. 持续更新进度
i-rs-read update "深入理解计算机系统" --current-page 100
i-rs-read update "深入理解计算机系统" --add-remark "第2章完成了"
i-rs-read update "深入理解计算机系统" --current-page 200
i-rs-read update "深入理解计算机系统" --add-remark "数据表示部分很有趣"

# 4. 中途暂停
i-rs-read update "深入理解计算机系统" --status paused

# 5. 继续阅读
i-rs-read update "深入理解计算机系统" --status reading
i-rs-read update "深入理解计算机系统" --current-page 400
i-rs-read update "深入理解计算机系统" --add-remark "终于到汇编了"

# 6. 完成阅读
i-rs-read update "深入理解计算机系统" --status completed --rating 5 \
  --review "这本书让我对计算机系统有了全新的认识，强烈推荐给所有计算机专业的学生！"

# 7. 查看统计
i-rs-read stats --tag cs
```

### 管理多个阅读计划

```bash
# 技术书籍阅读计划
i-rs-read add "Rust 权威指南" "Marcel自" 600 --tags rust,tech
i-rs-read add "Go 实战" "Alan A. A. Donovan" 480 --tags golang,tech
i-rs-read add "Python 编程" "Mark Lutz" 1600 --tags python,tech

# 非技术书籍
i-rs-read add "原子习惯" "James Clear" 320 --tags habit,non-fiction
i-rs-read add "深度工作" "Cal Newport" 230 --tags productivity,non-fiction

# 小说
i-rs-read add "三体" "刘慈欣" 890 --tags fiction,sci-fi

# 查看各类别
i-rs-read list --tag tech
i-rs-read list --tag non-fiction
i-rs-read list --tag fiction
```
