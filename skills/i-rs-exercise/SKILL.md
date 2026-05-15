---
name: "i-rs-exercise"
description: "运动记录追踪 CLI 工具，用于记录和管理健身活动。当用户需要追踪运动、记录健身、查看运动历史、显示统计或管理运动数据时调用。"
---

# i-rs-exercise

## Global Flags

- `--json` — Output in JSON format

运动记录追踪 CLI 工具，用于记录和管理健身活动。

## 存储位置

- 配置: `~/.config/i-rs/exercises.json`

## 命令

### add

添加新的运动记录。

```bash
i-rs-exercise add <NAME> <TYPE> <DURATION>
```

参数:
- `NAME` - 运动名称
- `TYPE` - 运动类型（如 running、swimming、gym、yoga、cycling、hiit）
- `DURATION` - 持续时间（分钟）

选项:
- `-c, --calories <CALORIES>` - 消耗卡路里
- `-t, --tag <TAG>` - 标签（可重复使用）
- `-n, --notes <NOTES>` - 备注（可重复使用）
- `-r, --remark <REMARK>` - 备注（可重复使用）

### list

列出运动记录。

```bash
i-rs-exercise list
```

选项:
- `-t, --tag <TAG>` - 按标签筛选
- `-y, --exercise-type <TYPE>` - 按运动类型筛选

### get

获取运动记录详情。

```bash
i-rs-exercise get <NAME>
```

### update

更新运动记录。

```bash
i-rs-exercise update <NAME>
```

选项:
- `-y, --exercise-type <TYPE>` - 新运动类型
- `-d, --duration-minutes <DURATION>` - 新持续时间
- `-c, --calories <CALORIES>` - 新卡路里值（空字符串清空）
- `-t, --tag <TAG>` - 新标签（覆盖）
- `-n, --notes <NOTES>` - 新备注
- `-r, --remark <REMARK>` - 新备注

### delete

删除运动记录。

```bash
i-rs-exercise delete <NAME>
```

### stats

显示运动统计数据。

```bash
i-rs-exercise stats
```

显示:
- 总记录数
- 总运动时长
- 总消耗卡路里
- 运动类型数量
- 最常见运动类型
- 时长最长运动类型
- 按类型分类的详细统计

## 示例

```bash
# 添加跑步记录

i-rs-exercise add "Morning Run" running 30 -c 300 -t morning -t cardio

# 添加健身房记录

i-rs-exercise add "Leg Day" gym 60 -c 500 -t strength -t legs

# 添加游泳记录

i-rs-exercise add "Lunch Swim" swimming 45 -c 400

# 列出所有记录

i-rs-exercise list

# 按标签筛选

i-rs-exercise list --tag cardio

# 按类型筛选

i-rs-exercise list --exercise-type running

# 获取详情

i-rs-exercise get "Morning Run"

# 更新记录

i-rs-exercise update "Morning Run" --duration-minutes 45 -c 350

# 查看统计

i-rs-exercise stats

# 删除记录

i-rs-exercise delete "Morning Run"

# JSON 输出

i-rs-exercise list --json
i-rs-exercise stats --json
```

### data

Manage data (export, import, clear).

```bash
i-rs-exercise data export
i-rs-exercise data import [FILE]
i-rs-exercise data clear
```

### example

Show usage examples.

```bash
i-rs-exercise example
```

### skill

Show skill information.

```bash
i-rs-exercise skill [summary|content|raw]
```

## Examples

```bash
# JSON output

i-rs-exercise list --json
```
