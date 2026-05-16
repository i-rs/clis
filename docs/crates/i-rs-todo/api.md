# i-rs-todo API

i-rs-todo 提供 REST API，可通过 `i-rs-api` 服务器访问。

## 启动 API 服务

```bash
i-rs-api
```

默认监听 `http://0.0.0.0:8080`，所有 todo 端点均在 `/todo` 路径下。

## 端点概览

| 方法 | 路径 | 描述 |
|------|------|------|
| GET | `/todo` | 列出所有待办事项 |
| POST | `/todo` | 添加待办事项 |
| GET | `/todo/{name}` | 获取单项详情 |
| PATCH | `/todo/{name}` | 更新待办事项 |
| POST | `/todo/{name}/done` | 标记完成/未完成 |
| DELETE | `/todo/{name}` | 删除待办事项 |

## 全局响应格式

所有成功响应统一格式：

```json
{
  "success": true,
  "data": ...,
  "meta": {
    "timestamp": "2026-01-15T10:30:00Z",
    "version": "0.0.2"
  }
}
```

列表端点额外包含 `meta.count`：

```json
{
  "success": true,
  "data": [...],
  "meta": {
    "count": 3,
    "timestamp": "2026-01-15T10:30:00Z",
    "version": "0.0.2"
  }
}
```

错误响应：

```json
{
  "success": false,
  "error": {
    "code": "NOT_FOUND",
    "message": "Todo 'xxx' not found"
  },
  "meta": {
    "timestamp": "2026-01-15T10:30:00Z",
    "version": "0.0.2"
  }
}
```

错误码：
- `NOT_FOUND` (404) — 条目不存在
- `CONFLICT` (409) — 条目已存在
- `BAD_REQUEST` (400) — 参数无效
- `SERVER_ERROR` (500) — 服务端错误

## 端点详情

### 列出待办事项

```
GET /todo
GET /todo?pending=true
GET /todo?done=true
GET /todo?tag=work
```

查询参数：
- `pending` (可选) — 仅显示未完成项
- `done` (可选) — 仅显示已完成项
- `tag` (可选) — 按标签过滤

### 添加待办事项

```
POST /todo
Content-Type: application/json

{
  "name": "buy-groceries",
  "title": "Buy groceries for the week",
  "priority": "high",
  "tag": ["shopping"],
  "content": ["Milk", "Eggs", "Bread"]
}
```

请求体：
- `name` (必填) — 唯一标识名
- `title` (可选) — 显示标题
- `priority` (可选) — 优先级 (high, medium, low)
- `tag` (可选) — 标签数组
- `content` (可选) — 内容列表

### 获取待办事项

```
GET /todo/{name}
```

路径参数：
- `name` — 待办事项标识名

### 更新待办事项

```
PATCH /todo/{name}
Content-Type: application/json

{
  "priority": "low",
  "tag": ["shopping", "urgent"]
}
```

路径参数：
- `name` — 待办事项标识名

请求体（所有字段可选）：
- `title` — 更新标题
- `priority` — 更新优先级
- `tag` — 替换标签
- `content` — 替换内容

### 标记完成/未完成

```
POST /todo/{name}/done
```

路径参数：
- `name` — 待办事项标识名

切换待办事项的完成状态。

### 删除待办事项

```
DELETE /todo/{name}
```

路径参数：
- `name` — 待办事项标识名
