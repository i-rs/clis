# i-rs-deploy 使用指南

## Global Flags

- `--json` — Output in JSON format

## 命令总览

| 命令 | 描述 |
|------|------|
| `add` | 添加部署记录 |
| `list` | 列出部署记录 |
| `get` | 查看部署详情 |
| `delete` | 删除部署记录 |
| `rollback` | 回滚到上一版本 |
| `stats` | 显示统计数据 |
| `example` | 显示使用示例 |
| `skill` | 查看 AI 技能文档 |

## add - 添加部署记录

```bash
i-rs-deploy add <PROJECT> <ENVIRONMENT> <VERSION> [OPTIONS]

# 示例
i-rs-deploy add myapp production v1.2.3 --status success --tag frontend
i-rs-deploy add myapp staging v1.2.4 --status failed --remark "Database timeout"
```

### 参数

- `PROJECT` - 项目名称
- `ENVIRONMENT` - 环境（production/staging/development）
- `VERSION` - 版本号

### 选项

| 选项 | 描述 | 默认值 |
|------|------|--------|
| `--status` | 部署状态 | success |
| `--rollback-from` | 回滚源记录ID | - |
| `-t, --tag` | 标签（可多次使用） | - |
| `-r, --remark` | 备注（可多次使用） | - |

### 部署状态

- `success` - 部署成功
- `failed` - 部署失败
- `rolling_back` - 回滚中
- `rolled_back` - 已回滚

## list - 列出部署记录

```bash
i-rs-deploy list [OPTIONS]

# 示例
i-rs-deploy list
i-rs-deploy list --project myapp
i-rs-deploy list --env production
i-rs-deploy list --tag frontend
```

### 选项

| 选项 | 描述 |
|------|------|
| `--project` | 按项目过滤 |
| `--env` | 按环境过滤 |
| `-t, --tag` | 按标签过滤 |
| `--json` | JSON 格式输出 |

## get - 查看部署详情

```bash
i-rs-deploy get <ID> [OPTIONS]

# 示例
i-rs-deploy get abc12345
i-rs-deploy get abc12345 --json
```

## delete - 删除部署记录

```bash
i-rs-deploy delete <ID>

# 示例
i-rs-deploy delete abc12345
```

## rollback - 回滚部署

```bash
i-rs-deploy rollback <PROJECT> <ENVIRONMENT> [OPTIONS]

# 示例
# 回滚到上一版本
i-rs-deploy rollback myapp production

# 回滚到指定版本
i-rs-deploy rollback myapp production --rollback-to abc12345
```

### 参数

- `PROJECT` - 项目名称
- `ENVIRONMENT` - 环境

### 选项

| 选项 | 描述 |
|------|------|
| `--rollback-to` | 指定回滚目标版本ID |

## stats - 显示统计数据

```bash
i-rs-deploy stats [OPTIONS]

# 示例
i-rs-deploy stats
i-rs-deploy stats --project myapp
i-rs-deploy stats --env production
```

### 选项

| 选项 | 描述 |
|------|------|
| `--project` | 按项目过滤 |
| `--env` | 按环境过滤 |

## example - 使用示例

```bash
i-rs-deploy example
```

## skill - AI 技能文档

```bash
i-rs-deploy skill         # 显示完整技能文档
i-rs-deploy skill summary # 显示摘要
i-rs-deploy skill content # 显示内容
```

## 全局选项

| 选项 | 描述 |
|------|------|
| `--json, -j` | JSON 格式输出 |
| `--help, -h` | 显示帮助 |

### data

Manage data (export, import, clear).

```bash
i-rs-deploy data export
i-rs-deploy data import [FILE]
i-rs-deploy data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
### example

Show usage examples.

```bash
i-rs-deploy example
```
### skill

Show skill information.

```bash
i-rs-deploy skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/deploy.json`
- Linux: `~/.config/i-rs/deploy.json`
- Windows: `~\AppData\Roaming\i-rs\deploy.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-deploy list
```
