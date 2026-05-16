# i-rs-sit API

i-rs-sit 提供 REST API，可通过 `i-rs-api` 服务器访问。

## 启动 API 服务

```bash
i-rs-api
```

默认监听 `http://0.0.0.0:8080`，所有 sit 端点均在 `/sit` 路径下。

## 端点概览

| 方法 | 路径 | 描述 |
|------|------|------|
| GET | `/sit` | 列出所有久坐记录 |
| POST | `/sit` | 添加久坐记录 |
| GET | `/sit/{id}` | 获取单条记录 |
| DELETE | `/sit/{id}` | 删除记录 |
| PATCH | `/sit/{id}` | 更新记录 |

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
    "message": "Sit 'uuid-xxx' not found"
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

### 列出记录

```
GET /sit
```

### 添加记录

```
POST /sit
Content-Type: application/json

{
  "duration_minutes": 45,
  "started_at": "2026-01-15T09:00:00Z",
  "ended_at": "2026-01-15T09:45:00Z",
  "tags": ["work"],
  "remark": []
}
```

请求体：
- `duration_minutes` (必填) — 久坐时长 (分钟)
- `started_at` (必填) — 开始时间 (RFC3339 格式)
- `ended_at` (必填) — 结束时间 (RFC3339 格式)
- `tags` (可选) — 标签数组
- `remark` (可选) — 备注数组

### 获取记录

```
GET /sit/{id}
```

路径参数：
- `id` — UUID 格式的条目 ID

### 更新记录

```
PATCH /sit/{id}
Content-Type: application/json

{
  "duration_minutes": 60
}
```

路径参数：
- `id` — UUID 格式的条目 ID

请求体（所有字段可选）：
- `duration_minutes` — 更新久坐时长
- `tags` — 替换标签
- `remark` — 替换备注

### 删除记录

```
DELETE /sit/{id}
```

路径参数：
- `id` — UUID 格式的条目 ID
