# i-rs-deploy 使用示例

## 基础操作

### 添加成功的部署记录

```bash
i-rs-deploy add myapp production v1.2.3 --status success --tag frontend
```

输出：
```
✓ Deploy record created: abc12345-def6-7890-ghij-klmnopqrstuv (myapp/production/v1.2.3)
```

### 添加失败的部署记录

```bash
i-rs-deploy add myapp staging v1.2.4 --status failed --tag backend --remark "Database connection timeout"
```

### 添加带备注的部署

```bash
i-rs-deploy add api-server production v2.0.0 --status success \
  --tag backend \
  --tag api \
  --remark "New authentication feature" \
  --remark "Breaking changes in /auth endpoint"
```

### 回滚记录

```bash
i-rs-deploy add myapp production v1.2.5 --status rolled_back --rollback-from abc12345 --tag rollback
```

## 查看部署

### 列出所有部署

```bash
i-rs-deploy list
```

输出：
```
┌───────┬─────────┬───────────┬─────────┬──────────┬────────────────────┬─────────┐
│ ID    │ PROJECT  │ ENV       │ VERSION │ STATUS   │ DEPLOYED_AT        │ TAGS    │
├───────┼─────────┼───────────┼─────────┼──────────┼────────────────────┼─────────┤
│ abc12 │ myapp    │ production│ v1.2.3  │ success  │ 2024-01-15 10:30   │ frontend│
│ def34 │ myapp    │ staging   │ v1.2.4  │ failed   │ 2024-01-14 15:20   │ backend │
└───────┴─────────┴───────────┴─────────┴──────────┴────────────────────┴─────────┘

Total: 2 records
```

### 按项目过滤

```bash
i-rs-deploy list --project myapp
```

### 按环境过滤

```bash
i-rs-deploy list --env production
```

### 按标签过滤

```bash
i-rs-deploy list --tag frontend
```

### 查看部署详情

```bash
i-rs-deploy get abc12345
```

输出：
```
Deploy Record
ID: abc12345-def6-7890-ghij-klmnopqrstuv
Project: myapp
Environment: production
Version: v1.2.3
Status: success
Deployed At: 2024-01-15 10:30:00
Tags: frontend
Remark: New login feature
Created: 2024-01-15 10:30:00
Updated: 2024-01-15 10:30:00
```

## 回滚操作

### 回滚到上一版本

```bash
i-rs-deploy rollback myapp production
```

输出：
```
✓ Rolled back to v1.2.2 (def67890)
```

### 回滚到指定版本

```bash
i-rs-deploy rollback myapp production --rollback-to def67890
```

## 统计功能

### 查看所有部署统计

```bash
i-rs-deploy stats
```

输出：
```
Deploy Statistics
Total Deployments: 15

By Status:
  - success: 12
  - failed: 2
  - rolled_back: 1

By Project:
  - myapp: 8
  - api-server: 5
  - web-frontend: 2

By Environment:
  - production: 10
  - staging: 5

Latest Success: myapp
  Version: v1.2.3
  Environment: production
  Deployed At: 2024-01-15 10:30
  Ago: 2 days ago

Rollback Count: 1
```

### 按项目查看统计

```bash
i-rs-deploy stats --project myapp
```

### 按环境查看统计

```bash
i-rs-deploy stats --env production
```

## JSON 输出

### JSON 格式列表

```bash
i-rs-deploy list --json
```

输出：
```json
{
  "success": true,
  "data": [
    {
      "id": "abc12345",
      "project": "myapp",
      "environment": "production",
      "version": "v1.2.3",
      "status": "success",
      "deployed_at": "2024-01-15 10:30:00",
      "tags": ["frontend"],
      "remark": [],
      "created_at": "2024-01-15 10:30:00",
      "updated_at": "2024-01-15 10:30:00"
    }
  ],
  "meta": {
    "count": 1,
    "filter": null
  }
}
```

### JSON 格式详情

```bash
i-rs-deploy get abc12345 --json
```

## 工作流程示例

### 日常部署记录

```bash
# 1. 部署前环境检查
i-rs-deploy add myapp staging v1.2.5 --status success --tag pre-release

# 2. 生产环境部署
i-rs-deploy add myapp production v1.2.5 --status success --tag release --remark "Hotfix release"

# 3. 查看最近部署
i-rs-deploy list --project myapp --env production | head -5

# 4. 如果出现问题，回滚
i-rs-deploy rollback myapp production

# 5. 查看统计数据
i-rs-deploy stats --project myapp
```

### 回滚工作流

```bash
# 1. 发现问题，添加失败记录
i-rs-deploy add myapp production v1.2.6 --status failed --remark "Memory leak detected"

# 2. 回滚到上一稳定版本
i-rs-deploy rollback myapp production

# 3. 验证回滚成功
i-rs-deploy list --project myapp --env production | head -3

# 4. 查看回滚统计
i-rs-deploy stats --project myapp | grep -i rollback
```
