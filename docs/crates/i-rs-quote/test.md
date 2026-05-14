# i-rs-quote 测试记录

本文档用于记录 i-rs-quote 的功能测试用例。

## 测试环境

- 操作系统: macOS / Linux / Windows
- 数据文件: `~/.config/i-rs/quotes.json`

## 功能测试

### 1. 添加语录

#### 基本添加

```bash
$ i-rs-quote add --content "Test quote 1"
✓ Quote 'xxx' added successfully
```

#### 完整信息

```bash
$ i-rs-quote add \
  --content "Stay hungry, stay foolish." \
  --author "Steve Jobs" \
  --source "Stanford Commencement Speech" \
  --tag inspiration \
  --tag tech
✓ Quote 'yyy' added successfully
```

#### 带备注

```bash
$ i-rs-quote add \
  --content "Test quote with remark" \
  --author "Test Author" \
  --tag test \
  --remark "This is a test" \
  --remark "Added for testing purposes"
✓ Quote 'zzz' added successfully
```

### 2. 列出语录

#### 列出所有

```bash
$ i-rs-quote list
┌──────┬─────────────────────────────────────────┬────────────┬─────────────┬────────────────────┐
│ ID   │ CONTENT                                 │ AUTHOR     │ TAGS        │ CREATED            │
├──────┼─────────────────────────────────────────┼────────────┼─────────────┼────────────────────┤
│ yyy  │ Stay hungry, stay foolish...            │ Steve Jobs │ inspiration │ 2024-01-15 10:30   │
│ zzz  │ Test quote with remark                 │ Test Auth… │ test        │ 2024-01-15 10:31   │
└──────┴─────────────────────────────────────────┴────────────┴─────────────┴────────────────────┘

Total: 2 quotes
```

#### 按标签筛选

```bash
$ i-rs-quote list --tag inspiration
┌──────┬─────────────────────────────────────────┬────────────┬─────────────┬────────────────────┐
│ ID   │ CONTENT                                 │ AUTHOR     │ TAGS        │ CREATED            │
├──────┼─────────────────────────────────────────┼────────────┼─────────────┼────────────────────┤
│ yyy  │ Stay hungry, stay foolish...            │ Steve Jobs │ inspiration │ 2024-01-15 10:30   │
└──────┴─────────────────────────────────────────┴────────────┴─────────────┴────────────────────┘

Total: 1 quotes
```

#### 按作者搜索

```bash
$ i-rs-quote list --author "Steve"
┌──────┬─────────────────────────────────────────┬────────────┬─────────────┬────────────────────┐
│ ID   │ CONTENT                                 │ AUTHOR     │ TAGS        │ CREATED            │
├──────┼─────────────────────────────────────────┼────────────┼─────────────┼────────────────────┤
│ yyy  │ Stay hungry, stay foolish...            │ Steve Jobs │ inspiration │ 2024-01-15 10:30   │
└──────┴─────────────────────────────────────────┴────────────┴─────────────┴────────────────────┘

Total: 1 quotes
```

### 3. 查看详情

```bash
$ i-rs-quote get yyy

Quote: yyy

Content: Stay hungry, stay foolish.

Author: Steve Jobs
Source: Stanford Commencement Speech
Tags: inspiration, tech

Created: 2024-01-15 10:30:00
```

### 4. 随机展示

```bash
$ i-rs-quote random

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  Stay hungry, stay foolish.

  — Steve Jobs
  (Stanford Commencement Speech)

  [inspiration] [tech]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  [yyy]
```

### 5. 删除语录

```bash
$ i-rs-quote delete zzz
✓ Quote 'zzz' deleted successfully
```

### 6. JSON 输出

#### 列表 JSON

```bash
$ i-rs-quote list --json
{
  "success": true,
  "data": [...],
  "meta": {
    "count": 1,
    "filter": "all"
  }
}
```

#### 详情 JSON

```bash
$ i-rs-quote get yyy --json
{
  "success": true,
  "data": {
    "id": "yyy",
    "content": "Stay hungry, stay foolish.",
    "author": "Steve Jobs",
    "source": "Stanford Commencement Speech",
    "tags": ["inspiration", "tech"],
    "remark": [],
    "created_at": "2024-01-15 10:30:00"
  }
}
```

### 7. 错误处理

#### 获取不存在的语录

```bash
$ i-rs-quote get nonexistent-id
Error: Quote 'nonexistent-id' not found
```

#### 删除不存在的语录

```bash
$ i-rs-quote delete nonexistent-id
Error: Quote 'nonexistent-id' not found
```

#### 空列表

```bash
$ i-rs-quote list --tag nonexistent
Warning: No quotes found.
```

## 边界测试

### 空内容

```bash
$ i-rs-quote add --content ""
✓ Quote '...' added successfully
```

### 特殊字符

```bash
$ i-rs-quote add --content "测试中文 & emoji 🎉 \"quotes\"" --author "测试"
✓ Quote '...' added successfully
```

### 大量标签

```bash
$ i-rs-quote add --content "Test" --tag tag1 --tag tag2 --tag tag3 --tag tag4 --tag tag5
✓ Quote '...' added successfully
```

### 长文本

```bash
$ i-rs-quote add --content "Lorem ipsum..." (1000+ characters)
✓ Quote '...' added successfully
```

## 性能测试

### 大量数据

```bash
# 添加 100 条语录
for i in {1..100}; do
  i-rs-quote add --content "Quote $i" --author "Author $i" --tag test
done

# 列出性能
$ time i-rs-quote list
```

## 数据持久化测试

### 重启后数据保留

```bash
# 添加数据
$ i-rs-quote add --content "Persistent quote" --author "Test"

# 重启终端/系统

# 验证数据存在
$ i-rs-quote list
# 应该看到 "Persistent quote"
```
