# i-rs-exercise

运动记录追踪 CLI 工具，用于记录和管理健身活动。

## 概述

i-rs-exercise 帮助您追踪日常运动记录，支持多种运动类型、卡路里追踪、标签管理和统计分析。适合健身爱好者、运动员和任何希望记录运动数据的人。

## 快速开始

```bash
# 添加运动记录
i-rs-exercise add "Morning Run" running 30 -c 300 -t morning -t cardio

# 列出所有运动记录
i-rs-exercise list

# 按标签筛选
i-rs-exercise list --tag cardio

# 按运动类型筛选
i-rs-exercise list --exercise-type running

# 查看统计信息
i-rs-exercise stats

# 获取详细记录
i-rs-exercise get "Morning Run"

# 更新记录
i-rs-exercise update "Morning Run" --duration-minutes 45

# 删除记录
i-rs-exercise delete "Morning Run"
```

## 安装

```bash
# npm
npm install -g @i-rs/i-rs-exercise

# 或 Homebrew
brew install i-rs/homebrew-tap/i-rs-exercise
```

## 数据存储

- macOS: `~/.config/i-rs/exercises.json`
- Linux: `~/.config/i-rs/exercises.json`
- Windows: `~\AppData\Roaming\i-rs\exercise.json`

## 功能特性

- **运动记录管理**：添加、查看、更新、删除运动记录
- **多种运动类型**：支持自定义运动类型（跑步、游泳、健身等）
- **卡路里追踪**：可选记录每次运动的卡路里消耗
- **标签支持**：使用标签组织和筛选运动记录
- **统计分析**：查看运动时长、卡路里消耗等统计数据

## 命令列表

- [使用说明](./usage.md) - 详细命令参考
- [使用示例](./examples.md) - 丰富的使用示例
- [测试记录](./test.md) - 测试数据
