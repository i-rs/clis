# i-rs-exercise 使用示例

## 基础使用

### 添加运动记录

```bash
# 简单跑步记录
i-rs-exercise add "Morning Run" running 30

# 带卡路里的游泳记录
i-rs-exercise add "Lunch Swim" swimming 45 -c 400

# 带多标签的健身房记录
i-rs-exercise add "Leg Day" gym 60 -c 500 -t legs -t strength

# 带备注的记录
i-rs-exercise add "Evening Yoga" yoga 30 -c 150 -n "Relaxing session" -r "Good stretch"
```

### 列出记录

```bash
# 列出所有记录
i-rs-exercise list

# 筛选特定标签
i-rs-exercise list --tag cardio
i-rs-exercise list --tag strength

# 筛选特定运动类型
i-rs-exercise list --exercise-type running
i-rs-exercise list --exercise-type gym
```

### 查看详情

```bash
# 查看单条记录详情
i-rs-exercise get "Morning Run"

# JSON 格式输出
i-rs-exercise get "Morning Run" --json
```

### 更新记录

```bash
# 更新持续时间
i-rs-exercise update "Morning Run" --duration-minutes 45

# 更新卡路里
i-rs-exercise update "Morning Run" --calories 350

# 更新标签
i-rs-exercise update "Morning Run" -t morning -t cardio -t interval

# 更新备注
i-rs-exercise update "Morning Run" -r "Great workout!"

# 一次性更新多个字段
i-rs-exercise update "Morning Run" -d 50 -c 400 -t cardio
```

### 删除记录

```bash
# 删除记录
i-rs-exercise delete "Morning Run"
```

---

## 高级使用

### 统计分析

```bash
# 查看完整统计
i-rs-exercise stats

# JSON 格式统计
i-rs-exercise stats --json
```

### 工作流程示例

#### 每日锻炼记录

```bash
# 早上跑步
i-rs-exercise add "Morning Run" running 30 -c 280 -t morning -t cardio

# 中午健身房
i-rs-exercise add "Lunch Gym" gym 45 -c 350 -t strength -t core

# 晚上瑜伽
i-rs-exercise add "Evening Yoga" yoga 20 -c 100 -t relaxation

# 查看今日汇总
i-rs-exercise list
```

#### 周度回顾

```bash
# 添加本周运动
i-rs-exercise add "Mon Run" running 25 -c 230 -t cardio
i-rs-exercise add "Tue Gym" gym 50 -c 400 -t strength
i-rs-exercise add "Wed Rest Day" walking 15 -c 100 -t light
i-rs-exercise add "Thu Swim" swimming 40 -c 380 -t cardio
i-rs-exercise add "Fri Gym" gym 55 -c 450 -t strength
i-rs-exercise add "Sat Long Run" running 60 -c 550 -t cardio
i-rs-exercise add "Sun Yoga" yoga 30 -c 120 -t relaxation

# 查看本周统计
i-rs-exercise stats

# 查看有氧运动汇总
i-rs-exercise list --tag cardio
```

#### 按运动类型管理

```bash
# 记录跑步训练
i-rs-exercise add "Easy Run" running 30 -c 280 -t cardio
i-rs-exercise add "Tempo Run" running 45 -c 420 -t cardio -t interval
i-rs-exercise add "Long Run" running 90 -c 800 -t cardio

# 记录力量训练
i-rs-exercise add "Push Day" gym 60 -c 450 -t strength -t upper-body
i-rs-exercise add "Pull Day" gym 60 -c 450 -t strength -t upper-body
i-rs-exercise add "Leg Day" gym 75 -c 550 -t strength -t legs

# 查看跑步统计
i-rs-exercise list --exercise-type running

# 查看力量训练统计
i-rs-exercise list --exercise-type gym

# 查看整体统计
i-rs-exercise stats
```

---

## 实用脚本

### 快速记录模板

```bash
# 快速晨跑
alias morning-run='i-rs-exercise add "$(date +"%Y-%m-%d Morning Run")" running 30 -t morning'

# 快速健身房
alias gym-session='i-rs-exercise add "$(date +"%Y-%m-%d Gym")" gym 60 -t strength'

# 快速查看统计
alias exercise-stats='i-rs-exercise stats'
```

### 使用 JSON 输出处理数据

```bash
# 导出所有数据为 JSON
i-rs-exercise list --json > exercises_backup.json

# 统计特定类型总数
i-rs-exercise list --exercise-type running --json | jq '.meta.count'

# 查看最消耗卡路里的运动
i-rs-exercise list --json | jq '.data[] | select(.calories != null) | {name, calories}' | head -20
```

---

## 提示与技巧

### 标签使用建议

1. **按时间段**：morning、afternoon、evening
2. **按强度**：light、moderate、intense
3. **按类型**：cardio、strength、flexibility
4. **按部位**：legs、arms、core、upper-body、lower-body

```bash
# 使用多个标签
i-rs-exercise add "HIIT Workout" hiit 30 -c 400 \
  -t morning \
  -t cardio \
  -t intense \
  -t full-body
```

### 记录备注的时机

- 运动感受（好/累/一般）
- 天气条件（室内/室外）
- 特殊成就（首次/个人记录）
- 身体状态（空腹/餐后）

```bash
# 详细备注
i-rs-exercise add "Marathon Training" running 120 \
  -c 1200 \
  -t long-run \
  -t cardio \
  -n "Training for spring marathon" \
  -r "Felt strong, good pace maintained"
```

---

## 环境变量

使用 `CONFIG_DIR` 临时切换数据目录：

```bash
# 使用测试目录
CONFIG_DIR=/tmp/test i-rs-exercise list

# 使用特定项目目录
CONFIG_DIR=~/projects/fitness i-rs-exercise stats
```
