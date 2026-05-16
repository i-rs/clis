# i-rs-water API

i-rs-water 提供 REST API，可通过 `i-rs-api` 服务器访问。

## 启动 API 服务

```bash
i-rs-api
```

默认监听 `http://0.0.0.0:8080`，所有 water 端点均在 `/water` 路径下。

## 端点概览

| 方法 | 路径 | 描述 |
|------|------|------|
| GET | `/water` | 列出所有饮水记录 |
| POST | `/water` | 添加饮水记录 |
| GET | `/water/{id}` | 获取单条记录 |
| DELETE | `/water/{id}` | 删除记录 |
| PATCH | `/water/{id}` | 更新记录 |

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
    "message": "Water 'uuid-xxx' not found"
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
GET /water
```

### 添加记录

```
POST /water
Content-Type: application/json

{
  "amount_ml": 300,
  "tags": ["morning"],
  "remark": []
}
```

请求体：
- `amount_ml` (必填) — 饮水量 (ml)
- `tags` (可选) — 标签数组
- `remark` (可选) — 备注数组

### 获取记录

```
GET /water/{id}
```

路径参数：
- `id` — UUID 格式的条目 ID

### 更新记录

```
PATCH /water/{id}
Content-Type: application/json

{
  "amount_ml": 500,
  "remark": ["after workout"]
}
```

路径参数：
- `id` — UUID 格式的条目 ID

请求体（所有字段可选）：
- `amount_ml` — 更新饮水量
- `tags` — 替换标签
- `remark` — 替换备注

### 删除记录

```
DELETE /water/{id}
```

路径参数：
- `id` — UUID 格式的条目 ID
