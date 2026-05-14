# i-rs-read 测试记录

本文档用于记录 i-rs-read 工具的功能测试，帮助验证工具是否正常工作。

## 测试环境

- **工具版本**: 最新版本
- **测试日期**: 2026-05-14
- **操作系统**: macOS / Linux / Windows

## 测试前准备

### 清理测试数据

```bash
# 如果之前有测试数据，可以先删除
# 注意：这会删除所有阅读记录
rm ~/.config/i-rs/read.json
```

## 功能测试

### 1. 添加书籍测试

```bash
# 测试基本添加
i-rs-read add "测试书籍1" "测试作者" 100

# 验证输出
# Book Added
# ✓ Name: 测试书籍1
# Author: 测试作者
# Total Pages: 100
# Status: ToRead
```

```bash
# 测试添加带标签的书籍
i-rs-read add "测试书籍2" "测试作者2" 200 --tags tag1,tag2

# 验证输出应包含成功信息
```

```bash
# 测试添加带备注的书籍
i-rs-read add "测试书籍3" "测试作者3" 300 --remark "这是一个测试备注"
```

```bash
# 测试重复添加（应报错）
i-rs-read add "测试书籍1" "测试作者" 100

# 预期输出
# Error: Book '测试书籍1' already exists
```

### 2. 列出书籍测试

```bash
# 测试列出所有书籍
i-rs-read list

# 预期输出应包含表格，显示所有添加的书籍
```

```bash
# 测试按标签筛选
i-rs-read list --tag tag1

# 预期输出只包含 tag1 标签的书籍
```

```bash
# 测试按状态筛选
i-rs-read list --status to_read
i-rs-read list --status reading
```

### 3. 获取书籍详情测试

```bash
# 测试获取书籍详情
i-rs-read get "测试书籍1"

# 预期输出应包含：
# - 书名、作者、总页数
# - 当前页数、状态
# - 标签、备注
# - 创建和更新时间
```

```bash
# 测试获取不存在的书籍（应报错）
i-rs-read get "不存在的书"

# 预期输出
# Error: Book '不存在的书' not found
```

### 4. 更新书籍测试

```bash
# 测试更新当前页数
i-rs-read update "测试书籍1" --current-page 50

# 预期输出应显示更新后的进度
```

```bash
# 测试更新状态为阅读中
i-rs-read update "测试书籍1" --status reading
```

```bash
# 测试更新状态为已完成
i-rs-read update "测试书籍2" --status completed
```

```bash
# 测试添加评分
i-rs-read update "测试书籍2" --rating 4.5
```

```bash
# 测试添加评论
i-rs-read update "测试书籍2" --review "这是一本很棒的书！"
```

```bash
# 测试更新标签
i-rs-read update "测试书籍1" --tags newtag,updated
```

```bash
# 测试添加备注
i-rs-read update "测试书籍1" --add-remark "新增测试备注"
```

```bash
# 测试删除备注
i-rs-read update "测试书籍1" --remove-remark 1
```

```bash
# 测试页数超限（应报错）
i-rs-read update "测试书籍1" --current-page 999

# 预期输出
# Error: Current page (999) cannot exceed total pages (100)
```

```bash
# 测试无效状态（应报错）
i-rs-read update "测试书籍1" --status invalid_status

# 预期输出
# Error: Invalid status 'invalid_status'. Valid options: reading, completed, paused, dropped, to_read
```

```bash
# 测试无效评分（应报错）
i-rs-read update "测试书籍1" --rating 10

# 预期输出
# Error: Rating must be between 0 and 5
```

### 5. 删除书籍测试

```bash
# 测试删除书籍
i-rs-read delete "测试书籍3"

# 预期输出应显示删除成功
```

```bash
# 测试删除不存在的书籍（应报错）
i-rs-read delete "不存在的书"

# 预期输出
# Error: Book '不存在的书' not found
```

### 6. 统计功能测试

```bash
# 测试查看统计
i-rs-read stats

# 预期输出应包含：
# - 总书籍数
# - 总页数
# - 已读页数及百分比
# - 各种状态的数量
# - 评分统计
```

```bash
# 测试按标签统计
i-rs-read stats --tag programming
```

### 7. 示例命令测试

```bash
# 测试查看示例
i-rs-read example

# 预期输出应显示多个使用示例
```

### 8. 技能文档测试

```bash
# 测试查看技能文档
i-rs-read skill

# 预期输出应显示 AI 技能文档
```

```bash
# 测试查看技能摘要
i-rs-read skill --summary

# 预期输出应显示简要摘要
```

### 9. JSON 输出测试

```bash
# 测试 JSON 格式列出
i-rs-read list --json

# 预期输出应为有效的 JSON 格式
```

```bash
# 测试 JSON 格式获取详情
i-rs-read get "测试书籍1" --json
```

```bash
# 测试 JSON 格式统计
i-rs-read stats --json
```

## 测试结果记录表

| 测试项 | 命令 | 状态 | 备注 |
|--------|------|------|------|
| 基本添加 | `i-rs-read add "书1" "作者" 100` | ☐ |
| 带标签添加 | `i-rs-read add "书2" "作者" 200 --tags tag1` | ☐ |
| 带备注添加 | `i-rs-read add "书3" "作者" 300 --remark "备注"` | ☐ |
| 重复添加 | `i-rs-read add "书1" "作者" 100` | ☐ |
| 列出全部 | `i-rs-read list` | ☐ |
| 按标签列出 | `i-rs-read list --tag tag1` | ☐ |
| 按状态列出 | `i-rs-read list --status reading` | ☐ |
| 获取详情 | `i-rs-read get "书1"` | ☐ |
| 获取不存在 | `i-rs-read get "不存在"` | ☐ |
| 更新页数 | `i-rs-read update "书1" --current-page 50` | ☐ |
| 更新状态 | `i-rs-read update "书1" --status reading` | ☐ |
| 添加评分 | `i-rs-read update "书1" --rating 5` | ☐ |
| 添加评论 | `i-rs-read update "书1" --review "评论"` | ☐ |
| 添加备注 | `i-rs-read update "书1" --add-remark "备注"` | ☐ |
| 删除备注 | `i-rs-read update "书1" --remove-remark 1` | ☐ |
| 删除书籍 | `i-rs-read delete "书3"` | ☐ |
| 查看统计 | `i-rs-read stats` | ☐ |
| 查看示例 | `i-rs-read example` | ☐ |
| JSON 列表 | `i-rs-read list --json` | ☐ |

## 测试完成确认

- [ ] 所有添加命令测试通过
- [ ] 所有列出命令测试通过
- [ ] 所有获取命令测试通过
- [ ] 所有更新命令测试通过
- [ ] 所有删除命令测试通过
- [ ] 统计功能测试通过
- [ ] 示例命令测试通过
- [ ] JSON 输出测试通过
- [ ] 错误处理测试通过

## 测试后清理

```bash
# 测试完成后可以保留数据用于日常使用
# 或者删除测试数据
rm ~/.config/i-rs/read.json
```
