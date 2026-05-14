# i-rs-exercise 测试记录

本文档用于记录功能测试用例，确保 CLI 正常工作。

## 测试环境

- macOS
- 数据目录: `~/.config/i-rs/exercises.json`

## 测试前准备

清理现有数据（可选）：

```bash
# 备份当前数据
cp ~/.config/i-rs/exercises.json ~/.config/i-rs/exercises_backup.json

# 或使用测试目录
CONFIG_DIR=/tmp/test-exercise i-rs-exercise list
```

---

## 功能测试

### 1. 添加记录测试

```bash
# 测试基本添加
i-rs-exercise add "Test Run" running 30 -c 280

# 测试带标签
i-rs-exercise add "Test Gym" gym 45 -c 350 -t strength -t upper-body

# 测试带备注
i-rs-exercise add "Test Swim" swimming 40 -c 380 -n "Lap swimming" -r "Good workout"

# 测试多标签
i-rs-exercise add "Test HIIT" hiit 25 -c 300 -t cardio -t intense -t morning
```

**预期结果**：
- 每条记录成功添加，显示 "✓ Exercise 'xxx' added"
- 无错误输出

---

### 2. 列表功能测试

```bash
# 测试列出所有
i-rs-exercise list

# 测试按标签筛选
i-rs-exercise list --tag cardio
i-rs-exercise list --tag strength

# 测试按类型筛选
i-rs-exercise list --exercise-type running
i-rs-exercise list --exercise-type gym

# 测试空结果筛选
i-rs-exercise list --tag nonexistent
```

**预期结果**：
- 正常显示记录表格
- 筛选正确返回结果
- 空筛选显示警告信息

---

### 3. 获取详情测试

```bash
# 测试获取存在的记录
i-rs-exercise get "Test Run"

# 测试获取不存在的记录
i-rs-exercise get "NonExistent"
```

**预期结果**：
- 存在的记录显示完整详情
- 不存在的记录显示错误信息

---

### 4. 更新功能测试

```bash
# 测试更新持续时间
i-rs-exercise update "Test Run" --duration-minutes 35

# 测试更新卡路里
i-rs-exercise update "Test Run" --calories 300

# 测试清空卡路里
i-rs-exercise update "Test Run" --calories ''

# 测试更新标签
i-rs-exercise update "Test Run" -t cardio -t morning

# 测试更新备注
i-rs-exercise update "Test Run" -r "Updated!"

# 测试批量更新
i-rs-exercise update "Test Run" -d 40 -c 320 -t cardio

# 测试更新不存在的记录
i-rs-exercise update "NonExistent" --duration-minutes 30
```

**预期结果**：
- 更新成功显示 "✓ Exercise 'xxx' updated"
- 验证更新后的值

---

### 5. 删除功能测试

```bash
# 测试删除存在的记录
i-rs-exercise delete "Test Swim"

# 测试删除不存在的记录
i-rs-exercise delete "NonExistent"
```

**预期结果**：
- 删除成功显示 "✓ Exercise 'xxx' deleted"
- 不存在的记录显示错误

---

### 6. 统计功能测试

```bash
# 测试基本统计
i-rs-exercise stats

# 添加更多记录后再次统计
i-rs-exercise add "Test Yoga" yoga 30 -c 120 -t relaxation
i-rs-exercise add "Test Cycling" cycling 60 -c 450 -t cardio

i-rs-exercise stats
```

**预期结果**：
- 显示总记录数、总时长、总卡路里
- 显示各类型分类统计
- 显示最常见和最长运动类型

---

### 7. JSON 输出测试

```bash
# 测试列表 JSON
i-rs-exercise list --json

# 测试详情 JSON
i-rs-exercise get "Test Run" --json

# 测试统计 JSON
i-rs-exercise stats --json

# 测试筛选 JSON
i-rs-exercise list --tag cardio --json
```

**预期结果**：
- 返回有效的 JSON 格式
- 包含 success、data、meta/error 字段

---

## 集成测试

### 完整工作流测试

```bash
# 1. 添加多条记录
i-rs-exercise add "Integration Run 1" running 30 -c 280 -t cardio
i-rs-exercise add "Integration Run 2" running 45 -c 420 -t cardio
i-rs-exercise add "Integration Gym 1" gym 60 -c 500 -t strength
i-rs-exercise add "Integration Swim 1" swimming 40 -c 380 -t cardio

# 2. 验证列表
i-rs-exercise list

# 3. 验证统计
i-rs-exercise stats

# 4. 更新一条记录
i-rs-exercise update "Integration Run 1" -d 35 -c 320

# 5. 获取详情确认更新
i-rs-exercise get "Integration Run 1"

# 6. 按标签筛选
i-rs-exercise list --tag cardio

# 7. 按类型筛选
i-rs-exercise list --exercise-type running

# 8. 删除所有测试记录
i-rs-exercise delete "Integration Run 1"
i-rs-exercise delete "Integration Run 2"
i-rs-exercise delete "Integration Gym 1"
i-rs-exercise delete "Integration Swim 1"

# 9. 验证删除结果
i-rs-exercise list
```

---

## 测试数据模板

快速添加测试数据：

```bash
# 清除并重建测试数据
CONFIG_DIR=/tmp/test-exercise bash -c '
  # 添加测试数据
  i-rs-exercise add "Mon Morning Run" running 30 -c 280 -t morning -t cardio
  i-rs-exercise add "Mon Lunch Gym" gym 45 -c 350 -t strength -t upper-body
  i-rs-exercise add "Tue Swim" swimming 40 -c 380 -t cardio
  i-rs-exercise add "Wed Yoga" yoga 30 -c 120 -t relaxation
  i-rs-exercise add "Thu HIIT" hiit 25 -c 300 -t cardio -t intense
  i-rs-exercise add "Fri Long Run" running 60 -c 550 -t cardio
  i-rs-exercise add "Sat Rest Day" walking 15 -c 100 -t light
  i-rs-exercise add "Sun Gym" gym 75 -c 600 -t strength -t full-body

  # 显示统计
  i-rs-exercise stats

  # 清理
  i-rs-exercise delete "Mon Morning Run"
  i-rs-exercise delete "Mon Lunch Gym"
  i-rs-exercise delete "Tue Swim"
  i-rs-exercise delete "Wed Yoga"
  i-rs-exercise delete "Thu HIIT"
  i-rs-exercise delete "Fri Long Run"
  i-rs-exercise delete "Sat Rest Day"
  i-rs-exercise delete "Sun Gym"
'
```

---

## 验收标准

完成测试后应满足：

- [ ] `add` 命令成功添加各种类型的记录
- [ ] `list` 命令正确显示所有记录
- [ ] `list` 命令按标签筛选正确
- [ ] `list` 命令按类型筛选正确
- [ ] `get` 命令正确显示单条记录详情
- [ ] `get` 命令处理不存在的记录
- [ ] `update` 命令正确更新各字段
- [ ] `update` 命令处理不存在的记录
- [ ] `delete` 命令正确删除记录
- [ ] `delete` 命令处理不存在的记录
- [ ] `stats` 命令正确计算统计数据
- [ ] `--json` 标志在所有命令中正常工作
- [ ] 错误消息清晰有用
