# i-rs-quote 使用示例

## 基础操作

### 添加简单语录

```bash
i-rs-quote add --content "Stay hungry, stay foolish."
```

### 添加完整信息语录

```bash
i-rs-quote add \
  --content "The only way to do great work is to love what you do." \
  --author "Steve Jobs" \
  --source "Stanford Commencement Speech, 2005" \
  --tag inspiration \
  --tag career
```

### 添加书籍摘录

```bash
i-rs-quote add \
  --content "It was the best of times, it was the worst of times." \
  --author "Charles Dickens" \
  --source "A Tale of Two Cities" \
  --tag literature \
  --tag classic \
  --remark "经典开篇"
```

### 添加诗歌节选

```bash
i-rs-quote add \
  --content "Two roads diverged in a wood, and I— / I took the one less traveled by" \
  --author "Robert Frost" \
  --source "The Road Not Taken" \
  --tag poetry \
  --tag life-choices
```

### 添加中文名言

```bash
i-rs-quote add \
  --content "天下熙熙，皆为利来；天下攘攘，皆为利往。" \
  --author "司马迁" \
  --source "史记·货殖列传" \
  --tag wisdom \
  --tag chinese \
  --tag classic
```

## 列表查询

### 列出所有语录

```bash
i-rs-quote list
```

### 按标签筛选

```bash
i-rs-quote list --tag inspiration
```

### 按作者搜索

```bash
i-rs-quote list --author "Steve"
```

### 模糊匹配作者

```bash
i-rs-quote list --author "Dickens"
```

## 查看详情

### 查看特定语录

```bash
i-rs-quote get 550e8400-e29b-41d4-a716-446655440000
```

## 随机展示

### 随机展示一条语录

```bash
i-rs-quote random
```

这个命令会以精美的格式随机展示一条语录，适合作为每日灵感。

## 删除操作

### 删除语录

```bash
i-rs-quote delete 550e8400-e29b-41d4-a716-446655440000
```

## 实际应用场景

### 收集演讲金句

```bash
# TED 演讲
i-rs-quote add --content "Your time is limited, don't waste it living someone else's life." --author "Steve Jobs" --source "Stanford Commencement Speech" --tag ted --tag inspiration

# 乔布斯传
i-rs-quote add --content "Innovation distinguishes between a leader and a follower." --author "Steve Jobs" --tag leadership --tag innovation
```

### 整理读书笔记

```bash
# 原则
i-rs-quote add --content "Truth is the essential foundation for any outcome." --author "Ray Dalio" --source "Principles" --tag book --tag truth

# 原子习惯
i-rs-quote add --content "You do not rise to the level of your goals. You fall to the level of your systems." --author "James Clear" --source "Atomic Habits" --tag habit --tag book
```

### 收藏哲理句子

```bash
# 苏格拉底
i-rs-quote add --content "The unexamined life is not worth living." --author "Socrates" --tag philosophy --tag wisdom

# 尼采
i-rs-quote add --content "He who has a why to live can bear almost any how." --author "Friedrich Nietzsche" --tag philosophy --tag motivation
```

### 记录电影台词

```bash
i-rs-quote add --content "Life is like a box of chocolates, you never know what you're gonna get." --author "Forrest Gump" --source "Forrest Gump" --tag movie --tag life

i-rs-quote add --content "Here's looking at you, kid." --author "Humphrey Bogart" --source "Casablanca" --tag movie --tag classic
```

## 批量操作脚本

### 批量导入语录

```bash
#!/bin/bash

# 批量添加经典语录
quotes=(
  "--content \"Knowledge is power.\" --author \"Francis Bacon\" --tag wisdom"
  "--content \"I think, therefore I am.\" --author \"René Descartes\" --tag philosophy"
  "--content \"To be or not to be, that is the question.\" --author \"William Shakespeare\" --tag literature"
)

for q in "${quotes[@]}"; do
  i-rs-quote add $q
done
```

### 备份和恢复

```bash
# 备份
cp ~/.config/i-rs/quotes.json ~/quotes_backup.json

# 恢复
cp ~/quotes_backup.json ~/.config/i-rs/quotes.json
```

## JSON 输出应用

### 程序化处理

```bash
# 获取所有标签
i-rs-quote list --json | jq '.[].data[].tags | .[]' | sort | uniq

# 统计各作者语录数量
i-rs-quote list --json | jq '.data[] | select(.author != null) | .author' | sort | uniq -c

# 导出到其他格式
i-rs-quote list --json | jq -r '.data[] | "\(.author // "-") | \(.content)"' > quotes.txt
```

## 提示和技巧

### 使用多标签组织

```bash
# 同一语录可以用多个标签
i-rs-quote add \
  --content "..." \
  --tag inspiration \
  --tag motivation \
  --tag morning \
  --tag daily
```

### 利用备注记录来源

```bash
# 添加详细备注
i-rs-quote add \
  --content "..." \
  --author "..." \
  --remark "2024年1月15日在播客中听到" \
  --remark "与朋友讨论时想起"
```

### 定期回顾

```bash
# 每天随机看一条
crontab -e
# 添加: 0 9 * * * /usr/local/bin/i-rs-quote random
```
