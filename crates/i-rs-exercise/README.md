# i-rs-exercise

运动记录追踪 CLI 工具，用于记录和管理健身活动。

## 功能特性

- **运动记录管理**：添加、查看、更新、删除运动记录
- **多种运动类型**：支持自定义运动类型（跑步、游泳、健身等）
- **卡路里追踪**：可选记录每次运动的卡路里消耗
- **标签支持**：使用标签组织和筛选运动记录
- **统计分析**：查看运动时长、卡路里消耗等统计数据
- **支持 JSON 输出**：所有命令支持 `--json` 全局标志

## 安装

```bash
npm install -g @i-rs/i-rs-exercise
# 或
brew install i-rs/homebrew-tap/i-rs-exercise
```

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

## 命令说明

### add

添加新的运动记录。

```bash
i-rs-exercise add <NAME> <TYPE> <DURATION> [OPTIONS]
```

参数：
- `NAME` - 运动名称
- `TYPE` - 运动类型（如 running、swimming、gym）
- `DURATION` - 持续时间（分钟）

选项：
- `-c, --calories <CALORIES>` - 消耗卡路里
- `-t, --tag <TAG>` - 标签（可多次使用）
- `-n, --notes <NOTES>` - 备注（可多次使用）
- `-r, --remark <REMARK>` - 备注（可多次使用）

### list

列出所有运动记录。

```bash
i-rs-exercise list [OPTIONS]
```

选项：
- `-t, --tag <TAG>` - 按标签筛选
- `-y, --exercise-type <TYPE>` - 按运动类型筛选

### get

获取运动记录的详细信息。

```bash
i-rs-exercise get <NAME>
```

### update

更新现有运动记录。

```bash
i-rs-exercise update <NAME> [OPTIONS]
```

选项：
- `-y, --exercise-type <TYPE>` - 新运动类型
- `-d, --duration-minutes <DURATION>` - 新持续时间
- `-c, --calories <CALORIES>` - 新卡路里值（使用 `--calories ''` 清空）
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

显示：
- 总记录数
- 总运动时长
- 总消耗卡路里
- 运动类型数量
- 最常见运动类型
- 时长最长运动类型
- 按类型分类的详细统计

### example

显示使用示例。

```bash
i-rs-exercise example
```

### skill

查看 AI 技能文档。

```bash
i-rs-exercise skill          # 显示完整技能文档
i-rs-exercise skill summary  # 显示摘要
i-rs-exercise skill content  # 显示内容
i-rs-exercise skill raw      # 显示原始文档
```

## 数据存储

配置存储在本地：
- macOS: `~/.config/i-rs/exercises.json`
- Linux: `~/.config/i-rs/exercises.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## 环境变量

- `CONFIG_DIR` - 覆盖配置目录路径

```bash
CONFIG_DIR=/tmp i-rs-exercise list
```

## JSON 输出

所有命令支持 `--json` 全局标志获取 JSON 格式输出：

```bash
i-rs-exercise list --json
i-rs-exercise stats --json
i-rs-exercise get "Morning Run" --json
```

## 许可

MIT OR Apache-2.0
