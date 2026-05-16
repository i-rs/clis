# i-rs-sleep API

i-rs-sleep 提供 REST API，可通过 `i-rs-api` 服务器访问。

## 启动 API 服务

```bash
i-rs-api
```

默认监听 `http://0.0.0.0:8080`，所有 sleep 端点均在 `/sleep` 路径下。

## 端点概览

| 方法 | 路径 | 描述 |
|------|------|------|
| GET | `/sleep` | 列出所有睡眠记录 |
| POST | `/sleep` | 添加睡眠记录 |
| GET | `/sleep/{id}` | 获取单条记录 |
| DELETE | `/sleep/{id}` | 删除记录 |
| PATCH | `/sleep/{id}` | 更新记录 |

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
    "message": "Sleep 'uuid-xxx' not found"
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
GET /sleep
```

### 添加记录

```
POST /sleep
Content-Type: application/json

{
  "bedtime": "2026-01-15T23:00:00Z",
  "wake_time": "2026-01-16T07:00:00Z",
  "quality": 4,
  "tags": ["weekday"],
  "remark": []
}
```

请求体：
- `bedtime` (必填) — 就寝时间 (RFC3339 格式)
- `wake_time` (必填) — 起床时间 (RFC3339 格式)
- `quality` (必填) — 睡眠质量评分 (1-5)
- `tags` (可选) — 标签数组
- `remark` (可选) — 备注数组

### 获取记录

```
GET /sleep/{id}
```

路径参数：
- `id` — UUID 格式的条目 ID

### 更新记录

```
PATCH /sleep/{id}
Content-Type: application/json

{
  "quality": 5
}
```

路径参数：
- `id` — UUID 格式的条目 ID

请求体（所有字段可选）：
- `bedtime` — 更新就寝时间
- `wake_time` — 更新起床时间
- `quality` — 更新质量评分
- `tags` — 替换标签
- `remark` — 替换备注

### 删除记录

```
DELETE /sleep/{id}
```

路径参数：
- `id` — UUID 格式的条目 ID
