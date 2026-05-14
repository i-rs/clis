# i-rs-deploy 测试记录

## 测试环境

- macOS
- i-rs-deploy v0.0.2

## 测试用例

### 1. 添加部署记录

```bash
# 添加成功部署
i-rs-deploy add myapp production v1.0.0 --status success --tag frontend
# 预期：✓ Deploy record created

i-rs-deploy add myapp staging v1.0.1 --status success --tag backend
# 预期：✓ Deploy record created

i-rs-deploy add myapp production v1.0.2 --status failed --tag frontend --remark "Build error"
# 预期：✓ Deploy record created

# 添加回滚记录
i-rs-deploy add api-server production v0.9.0 --status rolled_back --rollback-from test123
# 预期：✓ Deploy record created
```

### 2. 列出部署记录

```bash
# 列出所有
i-rs-deploy list
# 预期：显示所有记录，按时间倒序

# 按项目过滤
i-rs-deploy list --project myapp
# 预期：只显示 myapp 的记录

# 按环境过滤
i-rs-deploy list --env production
# 预期：只显示 production 环境的记录

# 按标签过滤
i-rs-deploy list --tag frontend
# 预期：只显示包含 frontend 标签的记录

# JSON 格式
i-rs-deploy list --json
# 预期：返回 JSON 格式数据
```

### 3. 查看部署详情

```bash
# 获取记录（使用部分 ID）
i-rs-deploy get abc123
# 预期：显示完整记录详情

# JSON 格式
i-rs-deploy get abc123 --json
# 预期：返回 JSON 格式数据

# 不存在的 ID
i-rs-deploy get nonexistent
# 预期：显示错误信息
```

### 4. 删除部署记录

```bash
# 删除记录
i-rs-deploy delete abc123
# 预期：✓ Deploy record deleted

# 删除不存在的记录
i-rs-deploy delete nonexistent
# 预期：显示错误信息
```

### 5. 回滚操作

```bash
# 回滚到上一版本
i-rs-deploy rollback myapp production
# 预期：✓ Rolled back to [version]

# 回滚到指定版本
i-rs-deploy rollback myapp production --rollback-to def456
# 预期：✓ Rolled back to [version]

# 回滚时无历史版本
i-rs-deploy rollback newapp production
# 预期：显示错误信息
```

### 6. 统计数据

```bash
# 所有统计
i-rs-deploy stats
# 预期：显示完整统计信息

# 按项目统计
i-rs-deploy stats --project myapp
# 预期：只显示 myapp 的统计

# 按环境统计
i-rs-deploy stats --env production
# 预期：只显示 production 的统计
```

### 7. 示例和技能文档

```bash
# 显示示例
i-rs-deploy example
# 预期：显示使用示例

# 显示技能文档摘要
i-rs-deploy skill summary
# 预期：显示摘要

# 显示技能文档内容
i-rs-deploy skill content
# 预期：显示详细命令说明

# 显示原始技能文档
i-rs-deploy skill
# 预期：显示完整原始格式
```

## 测试结果

| 测试项 | 状态 | 备注 |
|--------|------|------|
| 添加部署记录 | ✅ | |
| 列表查询 | ✅ | |
| 详情查看 | ✅ | |
| 删除记录 | ✅ | |
| 回滚操作 | ✅ | |
| 统计数据 | ✅ | |
| 示例显示 | ✅ | |
| 技能文档 | ✅ | |
| JSON 输出 | ✅ | |

## 回归测试

完成功能修改后，执行以下回归测试：

```bash
# 1. 基本 CRUD
i-rs-deploy add test-app production v0.0.1 --status success
i-rs-deploy list --project test-app
i-rs-deploy get $(i-rs-deploy list --project test-app --json | jq -r '.data[0].id')
i-rs-deploy delete $(i-rs-deploy list --project test-app --json | jq -r '.data[0].id')

# 2. 回滚功能
i-rs-deploy rollback myapp production
i-rs-deploy list --tag rollback

# 3. 统计功能
i-rs-deploy stats
i-rs-deploy stats --project myapp
```
