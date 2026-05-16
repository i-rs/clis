# i-rs-mood API

i-rs-mood 提供 REST API，可通过 `i-rs-api` 服务器访问。

## 启动 API 服务

```bash
i-rs-api
```

默认监听 `http://0.0.0.0:8080`，所有 mood 端点均在 `/mood` 路径下。

## 端点概览

| 方法 | 路径 | 描述 |
|------|------|------|
| GET | `/mood` | 列出心情记录 |
| POST | `/mood` | 添加心情记录 |
| GET | `/mood/stats` | 统计信息 |
| GET | `/mood/{date}` | 获取某天记录 |
| PATCH | `/mood/{date}` | 更新某天记录 |
| DELETE | `/mood/{date}` | 删除某天记录 |

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
    "message": "Mood '2026-01-15' not found"
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
GET /mood
GET /mood?days=7
```

查询参数：
- `days` (可选) — 返回最近 N 天的记录

### 添加记录

```
POST /mood
Content-Type: application/json

{
  "mood": "happy",
  "note": "Got a promotion!",
  "tag": ["work", "positive"]
}
```

请求体：
- `mood` (必填) — 心情标签 (如 happy, sad, anxious, calm)
- `note` (可选) — 心情备注
- `tag` (可选) — 标签数组

### 统计信息

```
GET /mood/stats
```

示例响应：

```json
{
  "success": true,
  "data": {
    "best": "happy",
    "worst": "anxious",
    "average": "3.5/5"
  }
}
```

### 获取记录

```
GET /mood/{date}
```

路径参数：
- `date` — 日期，格式 `YYYY-MM-DD`

### 更新记录

```
PATCH /mood/{date}
Content-Type: application/json

{
  "mood": "great"
}
```

路径参数：
- `date` — 日期，格式 `YYYY-MM-DD`

请求体（所有字段可选）：
- `mood` — 更新心情
- `note` — 更新备注
- `tag` — 替换标签

### 删除记录

```
DELETE /mood/{date}
```

路径参数：
- `date` — 日期，格式 `YYYY-MM-DD`
