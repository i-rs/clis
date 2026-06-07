# {工具名称} 使用引导

目标：让用户学会通过自然对话使用 {工具名称}

## 工具信息
- CLI 名称: `i-rs-xxx`
- 存储键: `"xxx"`
- 数据文件: `~/.i-rs/data/xxx.json`

## 命令列表
| 命令 | 参数 | 说明 |
|------|------|------|
| add | ... | ... |
| list | ... | ... |
| get | ... | ... |
| update | ... | ... |
| delete | ... | ... |

## 对话步骤设计原则
1. 步骤 1: 首次记录（add）
2. 步骤 2: 查询刚才的记录（get 或 list）
3. 步骤 3: 更新或修改（update）
4. 步骤 4: 特殊命令（如有：stats/done/calendar 等）
5. 步骤 5: 删除（delete）
6. 可选：带标签/备注的高级用法

## 扩展到 70 个 CLI
复制本目录，修改以下内容即可:
1. README.md 中的工具描述
2. script.json 中的 meta + steps
3. 根据实际 CLI subcommand 调整 expected_command 和 expected_args
